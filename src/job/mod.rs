use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
};

use tokio::sync::mpsc;
use tracing::debug;

use crate::JobReceiver;

use self::scan::scan_and_move;

mod scan;

#[derive(Debug)]
pub enum JobCommand {
    Enqueue(QueueKind),
    Clear,
}
#[derive(Debug)]
pub enum QueueKind {
    Scan(ScanQueueItem),
    Message(String),
}
#[derive(Debug)]
pub struct ScanQueueItem {
    pub path: PathBuf,
    pub retry_count: u8,
}

pub struct Queue {
    pub queue: Mutex<Vec<QueueKind>>,
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
    pub fn enqueue(&self, item: QueueKind) {
        self.queue.lock().unwrap().push(item);
        self.channel.send(()).unwrap();
    }
    pub fn dequeue(&self) -> Option<QueueKind> {
        self.queue.lock().unwrap().pop()
    }
    pub fn clear(&self) {
        self.queue.lock().unwrap().clear();
    }
}

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

    let semaphore = Arc::new(tokio::sync::Semaphore::new(10));

    while (receiver.recv().await).is_some() {
        let permit = semaphore.clone().acquire_owned().await.unwrap();
        let queue = queue.clone();
        tokio::spawn(async move {
            let _permit = permit;
            while let Some(item) = queue.dequeue() {
                match item {
                    QueueKind::Scan(item) => {
                        let res = scan_and_move(&item.path).await;
                        debug!("{:?}", &res);
                    }
                    QueueKind::Message(msg) => {
                        println!("Message: {:?}", msg);
                    }
                }
            }
        });
    }
}
