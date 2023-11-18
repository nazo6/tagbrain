use std::path::{Path, PathBuf};

use eyre::Context;
use tracing::warn;

use crate::config::CONFIG;
use crate::interface::metadata::Metadata;
use crate::job::utils::get_save_path_from_metadata;

use self::scan::ScanRes;

mod save;
mod scan;
mod utils;

pub struct ScanSuccessLog {
    pub old_metadata: Metadata,
    pub new_metadata: Metadata,
    pub scanner_info: ScannerInfo,
    pub target_path: PathBuf,
}
pub(crate) enum ScannerInfo {
    AcoustId { score: f64 },
    MusicbrainzSearch,
}

#[tracing::instrument]
pub(super) async fn scan_and_copy(path: &Path) -> eyre::Result<ScanSuccessLog> {
    let ScanRes {
        old_metadata,
        new_metadata,
        new_tag,
        scanner_info,
    } = scan::scan(path).await.wrap_err("Failed to scan")?;

    let new_path = get_save_path_from_metadata(path, &new_metadata)?;

    save::save_file(path, &new_path, new_tag)
        .await
        .wrap_err("Failed to save tag")?;

    if CONFIG.read().delete_original {
        let res = tokio::fs::remove_file(path).await;
        if let Err(e) = res {
            warn!("Failed to delete original file: {}", e);
        }
    }

    Ok(ScanSuccessLog {
        old_metadata,
        new_metadata,
        scanner_info,
        target_path: new_path,
    })
}
