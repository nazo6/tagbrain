use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
};

use tokio::sync::{mpsc, oneshot};
use tracing::info;

use crate::{config::CONFIG, JobReceiver};

mod scan_job;

#[derive(Debug, rspc::Type, serde::Serialize)]
pub struct QueueInfo {
    pub queue_count: u32,
    pub current_job: Option<JobTask>,
}

#[derive(Debug)]
pub enum JobCommand {
    Scan { path: PathBuf, retry_count: u8 },
    ScanAll,
    ClearQueue,
    GetQueueInfo { sender: oneshot::Sender<QueueInfo> },
}

#[derive(Debug, Clone, rspc::Type, serde::Serialize)]
pub enum JobTask {
    Scan { path: PathBuf, retry_count: u8 },
}

pub struct Queue {
    pub queue: Mutex<Vec<JobTask>>,
    pub current_job: Mutex<Option<JobTask>>,
    pub channel: mpsc::UnboundedSender<()>,
}
impl Queue {
    pub fn new() -> (Self, mpsc::UnboundedReceiver<()>) {
        let channel = mpsc::unbounded_channel();
        (
            Self {
                queue: Mutex::new(vec![]),
                current_job: Mutex::new(None),
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
        let job = self.queue.lock().unwrap().pop();
        *self.current_job.lock().unwrap() = job.clone();
        job
    }
    pub fn clear(&self) {
        self.queue.lock().unwrap().clear();
    }
}

#[tracing::instrument(skip(job_receiver))]
pub async fn start_job(mut job_receiver: JobReceiver) {
    let (queue, mut receiver) = Queue::new();
    let queue = Arc::new(queue);

    {
        let queue = queue.clone();
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
                                queue_count: queue.queue.lock().unwrap().len() as u32,
                                current_job: queue.current_job.lock().unwrap().clone(),
                            })
                            .unwrap();
                    }
                }
            }
        });
    }

    let semaphore = Arc::new(tokio::sync::Semaphore::new(1));

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
                }
            }
        }
    }
}
