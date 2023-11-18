use std::path::{Path, PathBuf};

use eyre::Context;
use lofty::TagExt;
use sqlx::query;
use tracing::{error, info, warn};

use crate::{
    api::musicbrainz::MusicbrainzClient,
    config::CONFIG,
    interface::{
        log::LogType,
        metadata::{write_metadata, Metadata},
    },
    POOL,
};

use super::utils::{
    fetch_cover_art, get_save_path_from_metadata, read_tag_or_default, response_to_metadata,
};

/// fix metadata with manually provided info
pub async fn fix_job(path: &Path, release_id: String, recording_id: String, copy_to_target: bool) {
    let res = fix_job_inner(path, release_id, recording_id, copy_to_target).await;
    match res {
        Ok(res) => {
            info!("Fix success: {}", path.display());
            let old_metadata = serde_json::to_string(&res.old_metadata).unwrap();
            let new_metadata = serde_json::to_string(&res.new_metadata).unwrap();
            let source_path = path.to_string_lossy();
            let target_path = res.target_path.to_string_lossy();
            let res = query!(
                "INSERT INTO log (success, type, message, old_metadata, new_metadata, source_path, target_path) VALUES (?,?,?,?,?,?,?)",
                true,
                LogType::Fix,
                "",
                old_metadata,
                new_metadata,
                source_path,
                target_path,
                ).execute(&*POOL).await;
            if let Err(err) = res {
                error!("Failed to insert log: {:?}", err);
            }
        }
        Err(e) => {
            let msg = format!("{:?}", e);
            let source = path.to_string_lossy();
            let res = query!(
                "INSERT INTO log (success, type, message, source_path) VALUES (?,?,?,?)",
                true,
                LogType::Fix,
                msg,
                source,
            )
            .execute(&*POOL)
            .await;
            tracing::error!("Failed to fix file: {}", e);
            if let Err(err) = res {
                error!("Failed to insert log: {:?}", err);
            }
        }
    }
}

struct FixJobRes {
    old_metadata: Metadata,
    new_metadata: Metadata,
    target_path: PathBuf,
}
#[tracing::instrument(err)]
async fn fix_job_inner(
    path: &Path,
    release_id: String,
    recording_id: String,
    copy_to_target: bool,
) -> eyre::Result<FixJobRes> {
    let mut tag = read_tag_or_default(path)?;
    let mb = MusicbrainzClient::new();
    let release = mb.release(&release_id).await?;
    let recording = mb.recording(&recording_id).await?;
    let metadata = response_to_metadata(recording, release)?;

    let new_path = get_save_path_from_metadata(path, &metadata)?;
    tokio::fs::create_dir_all(new_path.parent().unwrap()).await?;
    tokio::fs::copy(path, &new_path).await?;

    if !copy_to_target || CONFIG.read().delete_original {
        tokio::fs::remove_file(path).await?;
    }

    if tag.picture_count() == 0 {
        let cover_art = fetch_cover_art(&release_id).await;

        match cover_art {
            Ok(cover_art) => tag.push_picture(cover_art),
            Err(e) => warn!("Failed to fetch cover art: {}", e),
        }
    }

    write_metadata(&mut tag, metadata.clone());
    tag.save_to_path(new_path.clone())
        .wrap_err("Failed to write tag")?;

    let old_metadata = Metadata::from_tag(&tag);

    Ok(FixJobRes {
        old_metadata,
        new_metadata: metadata,
        target_path: new_path,
    })
}
