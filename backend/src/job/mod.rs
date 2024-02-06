use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
};

use tokio::sync::{mpsc, oneshot};
use tracing::info;

use crate::{config::CONFIG, JobReceiver};

mod fix_job;
mod scan_job;
mod utils;

#[derive(Debug)]
pub struct QueueInfo {
    pub tasks: Vec<JobTask>,
    pub running_count: usize,
}

#[derive(Debug)]
pub enum JobCommand {
    Scan {
        path: PathBuf,
        retry_count: u8,
    },
    ScanAll,
    ClearQueue,
    GetQueueInfo {
        sender: oneshot::Sender<QueueInfo>,
    },
    Fix {
        target_path: PathBuf,
        release_id: String,
        recording_id: String,
    },
    FixFailed {
        source_path: PathBuf,
        release_id: String,
        recording_id: String,
    },
}

#[derive(Debug, Clone, rspc::Type, serde::Serialize)]
pub enum JobTask {
    Scan {
        path: PathBuf,
        retry_count: u8,
    },
    Fix {
        path: PathBuf,
        release_id: String,
        recording_id: String,
        copy_to_target: bool,
    },
}

pub struct Queue {
    pub queue: Mutex<Vec<JobTask>>,
    pub channel: mpsc::UnboundedSender<()>,
}
impl Queue {
    pub fn new() -> (Self, mpsc::UnboundedReceiver<()>) {
        let channel = mpsc::unbounded_channel();
        (
            Self {
                queue: Mutex::new(vec![]),
                channel: channel.0,
            },
            channel.1,
        )
    }
    pub fn enqueue(&self, item: JobTask) {
        info!("Enqueue: {:?}", item);
        self.queue.lock().unwrap().push(item);
        self.channel.send(()).unwrap();
    }
    pub fn dequeue(&self) -> Option<JobTask> {
        self.queue.lock().unwrap().pop()
    }
    pub fn clear(&self) {
        self.queue.lock().unwrap().clear();
    }
}

#[tracing::instrument(skip(job_receiver))]
pub async fn start_job(mut job_receiver: JobReceiver) {
    let (queue, mut receiver) = Queue::new();
    let queue = Arc::new(queue);

    let semaphore = Arc::new(tokio::sync::Semaphore::new(1));

    {
        let queue = queue.clone();
        let semaphore = semaphore.clone();
        tokio::spawn(async move {
            while let Some(job) = job_receiver.recv().await {
                match job {
                    JobCommand::Scan { path, retry_count } => {
                        queue.enqueue(JobTask::Scan { path, retry_count });
                    }
                    JobCommand::ScanAll => {
                        let source_dir = CONFIG.read().source_dir.clone();
                        for item in walkdir::WalkDir::new(source_dir).into_iter().flatten() {
                            if item.file_type().is_file() {
                                queue.enqueue(JobTask::Scan {
                                    path: item.path().to_path_buf(),
                                    retry_count: 0,
                                });
                            }
                        }
                    }
                    JobCommand::ClearQueue => {
                        queue.clear();
                    }
                    JobCommand::GetQueueInfo { sender } => {
                        sender
                            .send(QueueInfo {
                                tasks: queue.queue.lock().unwrap().clone(),
                                running_count: 1 - semaphore.available_permits(),
                            })
                            .unwrap();
                    }
                    JobCommand::Fix {
                        target_path,
                        release_id,
                        recording_id,
                    } => {
                        queue.enqueue(JobTask::Fix {
                            path: target_path,
                            release_id,
                            recording_id,
                            copy_to_target: false,
                        });
                    }
                    JobCommand::FixFailed {
                        source_path,
                        release_id,
                        recording_id,
                    } => {
                        queue.enqueue(JobTask::Fix {
                            path: source_path,
                            release_id,
                            recording_id,
                            copy_to_target: true,
                        });
                    }
                }
            }
        });
    }

    loop {
        if receiver.recv().await.is_some() {
            while let Some(item) = queue.dequeue() {
                let semaphore = semaphore.clone();
                let permit = semaphore.clone().acquire_owned().await.unwrap();
                match item {
                    JobTask::Scan { path, retry_count } => {
                        let q2 = queue.clone();
                        tokio::spawn(async move {
                            let _permit = permit;
                            scan_job::scan_job(&path, q2, retry_count).await;
                        });
                    }
                    JobTask::Fix {
                        path,
                        release_id,
                        recording_id,
                        copy_to_target,
                    } => {
                        tokio::spawn(async move {
                            let _permit = permit;
                            fix_job::fix_job(&path, release_id, recording_id, copy_to_target).await;
                        });
                    }
                }
            }
        }
    }
}
