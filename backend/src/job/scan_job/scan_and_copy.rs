use std::path::{Path, PathBuf};

use eyre::{eyre, Context};
use tracing::warn;

use crate::config::CONFIG;
use crate::interface::metadata::Metadata;

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
        old_tag,
        scanner_info,
    } = scan::scan(path).await.wrap_err("Failed to scan")?;

    let Some(Some(ext)) = path.extension().map(|ext| ext.to_str()) else {
        return Err(eyre!("No extension found!"));
    };

    let new_path = {
        let mut new_path = PathBuf::new();

        let artist = new_metadata
            .artist
            .as_ref()
            .ok_or_else(|| eyre!("artist not found"))?;
        let album = new_metadata
            .album
            .as_ref()
            .ok_or_else(|| eyre!("album not found"))?;
        let title = new_metadata
            .title
            .as_ref()
            .ok_or_else(|| eyre!("title not found"))?;
        let album_artist = &new_metadata.album_artist;
        let track = &new_metadata.track;

        new_path.push(CONFIG.read().target_dir.clone());
        new_path.push(album_artist.clone().unwrap_or(artist.clone()));
        new_path.push(album.clone());
        let file_name = {
            let mut file_name = String::new();
            if let Some(track) = track {
                file_name.push_str(track);
                file_name.push_str(" - ");
            }
            file_name.push_str(title);
            file_name.push('.');
            file_name.push_str(ext);
            file_name
        };
        new_path.push(file_name);
        new_path
    };

    save::save_file(path, &new_path, old_tag, new_metadata.clone())
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
