use std::{path::Path, sync::Arc};

use sqlx::query;
use tracing::{error, info, warn};

use crate::{
    config::CONFIG,
    interface::log::LogType,
    job::{scan_job::scan_and_copy::scan_and_copy, JobTask},
    POOL,
};

mod scan_and_copy;

#[tracing::instrument(skip(queue))]
pub async fn scan_job(path: &Path, queue: Arc<crate::job::Queue>, retry_count: u8) {
    if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
        if CONFIG.read().allowed_extensions.iter().any(|e| e == ext) {
            let res = scan_and_copy(path).await;
            match res {
                Ok(res) => {
                    info!("Finished scanning: {}", path.display());
                    let old_metadata = serde_json::to_string(&res.old_metadata).unwrap();
                    let new_metadata = serde_json::to_string(&res.new_metadata).unwrap();
                    let source_path = path.to_string_lossy();
                    let target_path = res.target_path.to_string_lossy();
                    let (acoustid_score, message) = match res.scanner_info {
                        scan_and_copy::ScannerInfo::AcoustId { score } => {
                            (Some(score), "Scanner: AcoustId")
                        }
                        _ => (None, "Scanner: MusicBrainz Search"),
                    };
                    let res = query!(
                        "INSERT INTO log (success, type, message, old_metadata, new_metadata, source_path, target_path, acoustid_score, retry_count) VALUES (?,?,?,?,?,?,?,?,?)",
                        true,
                        LogType::Scan,
                        message,
                        old_metadata,
                        new_metadata,
                        source_path,
                        target_path,
                        acoustid_score,
                        retry_count
                    ).execute(&*POOL).await;
                    if let Err(err) = res {
                        error!("Failed to insert log: {:?}", err);
                    }
                }
                Err(err) => {
                    if retry_count < 1 {
                        warn!("Failed to scan. Retrying...: {:?}", err);
                        queue.enqueue(JobTask::Scan {
                            path: path.to_path_buf(),
                            retry_count: retry_count + 1,
                        });
                    } else {
                        error!("Failed to scan: {:?}", err);
                        let err = format!("{:?}", err);
                        let path = path.to_string_lossy();
                        query!(
                            "INSERT INTO log (success, type, message, source_path, retry_count) VALUES (?,?,?,?,?)",
                            false,
                            LogType::Scan,
                            err,
                            path,
                            retry_count
                        )
                        .execute(&*POOL)
                        .await
                        .unwrap();
                    }
                }
            }
            return;
        }
    }
    info!("Skipping: {} (not allowed extension)", path.display());
}
