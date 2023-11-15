use std::{path::Path, sync::Arc, time::Duration};

use tracing::{error, info, warn};

use crate::{
    config::CONFIG,
    job::{scan_job::scan::scan_and_copy, QueueItem, QueueKind},
};

mod scan;

#[tracing::instrument(skip(semaphore, queue))]
pub async fn scan_job(
    path: &Path,
    semaphore: Arc<tokio::sync::Semaphore>,
    queue: Arc<crate::job::Queue>,
    retry_count: u8,
) {
    if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
        if CONFIG.read().allowed_extensions.iter().any(|e| e == ext) {
            let _permit = semaphore.acquire_owned().await.unwrap();
            let res = scan_and_copy(path).await;
            tokio::time::sleep(Duration::from_secs(1)).await;
            if let Err(err) = res {
                if retry_count < 1 {
                    warn!("Failed to scan. Retrying...: {:?}", err);
                    queue.enqueue(QueueItem {
                        kind: QueueKind::Scan {
                            path: path.to_path_buf(),
                        },
                        retry_count: retry_count + 1,
                    });
                } else {
                    error!("Failed to scan: {:?}", err);
                }
            } else {
                info!("Finished scanning: {}", path.display());
            }

            return;
        }
    }
    info!("Skipping: {} (not allowed extension)", path.display());
}
