use std::collections::HashMap;
use std::path::Path;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::SystemTime;

use notify::event::ModifyKind;
use notify::{
    event::AccessKind, Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher,
};
use tokio::sync::mpsc::{channel, Receiver};
use tokio::time::{sleep, Duration};
use tracing::{info, warn};

use crate::config::CONFIG;
use crate::{job::JobCommand, JobSender};

pub fn async_watcher() -> notify::Result<(RecommendedWatcher, Receiver<notify::Result<Event>>)> {
    let (tx, rx) = channel(1);

    let watcher =
        RecommendedWatcher::new(move |res| tx.blocking_send(res).unwrap(), Config::default())?;

    Ok((watcher, rx))
}

#[tracing::instrument(skip(job_sender), err)]
pub async fn start_watcher(job_sender: JobSender) -> notify::Result<()> {
    let (mut watcher, mut rx) = async_watcher()?;

    let source_path = CONFIG.read().source_dir.clone();
    let source_dir = Path::new(&source_path);

    watcher.watch(source_dir, RecursiveMode::Recursive)?;

    let file_last_modified = Arc::new(Mutex::new(HashMap::<PathBuf, SystemTime>::new()));
    while let Some(res) = rx.recv().await {
        let file_last_modified = file_last_modified.clone();
        match res {
            Ok(event) => {
                if let EventKind::Access(AccessKind::Close(_)) | EventKind::Create(_) = event.kind {
                    if let Some(path) = event.paths.first() {
                        let path = path.to_path_buf();
                        let job_sender = job_sender.clone();
                        tokio::spawn(async move {
                            sleep(Duration::from_secs(1)).await;
                            if let Some(last_modified) =
                                file_last_modified.lock().unwrap().get(&path)
                            {
                                if last_modified.elapsed().unwrap().as_secs() < 1 {
                                    return;
                                }
                            }

                            info!("File added. Sending job to queue: {:?}", path);
                            job_sender
                                .send(JobCommand::Scan {
                                    path,
                                    retry_count: 0,
                                })
                                .unwrap();
                        });
                    }
                } else if let EventKind::Modify(ModifyKind::Name(notify::event::RenameMode::To)) =
                    event.kind
                {
                    // this occurs when a file is moved
                    if let Some(path) = event.paths.first() {
                        let path = path.to_path_buf();
                        let paths = if path.is_dir() {
                            walkdir::WalkDir::new(source_dir)
                                .into_iter()
                                .flatten()
                                .filter(|item| item.file_type().is_file())
                                .map(|item| item.path().to_path_buf())
                                .collect::<Vec<_>>()
                        } else {
                            vec![path]
                        };
                        paths.into_iter().for_each(|path| {
                            let job_sender = job_sender.clone();
                            let file_last_modified = file_last_modified.clone();
                            tokio::spawn(async move {
                                sleep(Duration::from_secs(1)).await;
                                if let Some(last_modified) =
                                    file_last_modified.lock().unwrap().get(&path)
                                {
                                    if last_modified.elapsed().unwrap().as_secs() < 1 {
                                        return;
                                    }
                                }

                                info!("File added. Sending job to queue: {:?}", path);
                                job_sender
                                    .send(JobCommand::Scan {
                                        path,
                                        retry_count: 0,
                                    })
                                    .unwrap();
                            });
                        })
                    }
                } else if let EventKind::Modify(_) = event.kind {
                    if let Some(path) = event.paths.first() {
                        file_last_modified
                            .lock()
                            .unwrap()
                            .insert(path.to_path_buf(), SystemTime::now());
                    }
                }
            }
            Err(e) => warn!("watch error: {:?}", e),
        }
    }
    Ok(())
}
