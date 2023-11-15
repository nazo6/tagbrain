use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
};

use tokio::sync::mpsc;

use crate::JobReceiver;

mod scan_job;

#[derive(Debug)]
pub enum JobCommand {
    Enqueue(QueueItem),
    Clear,
}
#[derive(Debug)]
pub struct QueueItem {
    pub kind: QueueKind,
    pub retry_count: u8,
}
#[derive(Debug)]
pub enum QueueKind {
    Scan { path: PathBuf },
}

pub struct Queue {
    pub queue: Mutex<Vec<QueueItem>>,
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
    pub fn enqueue(&self, item: QueueItem) {
        self.queue.lock().unwrap().push(item);
        self.channel.send(()).unwrap();
    }
    pub fn dequeue(&self) -> Option<QueueItem> {
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

    {
        let queue = queue.clone();
        tokio::spawn(async move {
            while let Some(job) = job_receiver.recv().await {
                match job {
                    JobCommand::Enqueue(item) => {
                        queue.enqueue(item);
                    }
                    JobCommand::Clear => {
                        queue.clear();
                    }
                }
            }
        });
    }

    let semaphore = Arc::new(tokio::sync::Semaphore::new(1));

    while (receiver.recv().await).is_some() {
        let queue = queue.clone();
        let semaphore = semaphore.clone();
        tokio::spawn(async move {
            while let Some(item) = queue.dequeue() {
                let semaphore = semaphore.clone();
                match item.kind {
                    QueueKind::Scan { path } => {
                        scan_job::scan_job(&path, semaphore, queue.clone(), item.retry_count).await;
                    }
                }
            }
        });
    }
}
