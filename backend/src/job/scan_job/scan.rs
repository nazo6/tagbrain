use std::path::{Path, PathBuf};

use eyre::{eyre, Context};

use crate::config::CONFIG;
use crate::interface::metadata::Metadata;

use self::scan::ScanRes;

mod save;
mod scan;
mod utils;

pub struct ScanSuccessLog {
    pub old_metadata: Metadata,
    pub new_metadata: Metadata,
    pub acoustid_score: f64,
    pub target_path: PathBuf,
}

#[tracing::instrument]
pub(super) async fn scan_and_copy(path: &Path) -> eyre::Result<ScanSuccessLog> {
    let ScanRes {
        old_metadata,
        new_metadata,
        acoustid_score,
        old_tag,
    } = scan::scan(path).await.wrap_err("Failed to scan")?;

    let Some(Some(ext)) = path.extension().map(|ext| ext.to_str()) else {
        return Err(eyre!("No extension found!"));
    };

    let new_path = {
        let mut new_path = PathBuf::new();
        new_path.push(CONFIG.read().target_dir.clone());
        new_path.push(
            new_metadata
                .album_artist
                .clone()
                .expect("album artist not found"),
        );
        new_path.push(new_metadata.album.clone().expect("album not found"));
        new_path.push(format!(
            "{} - {}.{}",
            new_metadata.track.as_ref().expect("track not found"),
            new_metadata.title.as_ref().expect("title not found"),
            ext
        ));
        new_path
    };

    save::save_file(path, &new_path, old_tag, new_metadata.clone())
        .await
        .wrap_err("Failed to save tag")?;

    Ok(ScanSuccessLog {
        old_metadata,
        new_metadata,
        acoustid_score,
        target_path: new_path,
    })
}
