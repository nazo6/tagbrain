use std::path::Path;

use eyre::eyre;
use lofty::Tag;
use tracing::{info, warn};

use crate::{
    api::musicbrainz::{recording::RecordingRes, MusicbrainzClient},
    interface::metadata::{write_metadata, Metadata},
    job::{
        scan_job::scan_and_copy::utils::find_best_release_and_recording,
        utils::{fetch_cover_art, read_tag_or_default, response_to_metadata},
    },
};

use super::ScannerInfo;

mod acoustid_scanner;
mod musicbrainz_search_scanner;

pub(super) struct ScanRes {
    pub old_metadata: Metadata,
    pub new_metadata: Metadata,
    pub new_tag: Tag,
    pub scanner_info: ScannerInfo,
}
struct ScannerRes {
    log: ScannerInfo,
    recordings: Vec<RecordingRes>,
}

#[tracing::instrument]
pub(super) async fn scan(path: &Path) -> eyre::Result<ScanRes> {
    let mut tag = read_tag_or_default(path)?;

    let scanner_res = if let Ok(res) = acoustid_scanner::acoustid_scanner(path).await {
        res
    } else {
        info!("Acoustid scanner failed. Falling back to musicbrainz search scanner.");
        musicbrainz_search_scanner::musicbrainz_search_scanner(&tag).await?
    };

    let (best_recording, best_release, best_score) = {
        find_best_release_and_recording(scanner_res.recordings, &tag)
            .ok_or_else(|| eyre!("Failed to find best match"))?
    };

    let mb_client = MusicbrainzClient::new();
    let release = mb_client.release(&best_release.id).await?;

    info!(
        "Best match release/recording was '{}({})' / '{}({})' with score {}",
        release.release_group.title,
        best_release.id,
        best_recording.title,
        best_recording.id,
        best_score
    );

    let old_metadata = Metadata::from_tag(&tag);

    if tag.picture_count() == 0 {
        let cover_art = fetch_cover_art(&release.id).await;

        match cover_art {
            Ok(cover_art) => tag.push_picture(cover_art),
            Err(e) => warn!("Failed to fetch cover art: {}", e),
        }
    }

    let new_metadata = response_to_metadata(best_recording, release)?;
    write_metadata(&mut tag, new_metadata.clone());

    Ok(ScanRes {
        old_metadata,
        new_metadata,
        new_tag: tag,
        scanner_info: scanner_res.log,
    })
}
