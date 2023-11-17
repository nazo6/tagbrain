use std::path::Path;

use eyre::{eyre, Context};
use lofty::{read_from_path, Tag, TaggedFileExt};
use tracing::info;

use crate::{
    api::musicbrainz::{recording::RecordingRes, MusicbrainzClient},
    interface::metadata::Metadata,
    job::scan_job::scan_and_copy::{
        scan::response_to_metadata::format_to_metadata, utils::find_best_release_and_recording,
    },
};

use super::ScannerInfo;

mod acoustid_scanner;
mod musicbrainz_search_scanner;
mod response_to_metadata;

pub(super) struct ScanRes {
    pub old_metadata: Metadata,
    pub new_metadata: Metadata,
    pub old_tag: Tag,
    pub scanner_info: ScannerInfo,
}
struct ScannerRes {
    log: ScannerInfo,
    recordings: Vec<RecordingRes>,
}

pub(super) async fn scan(path: &Path) -> eyre::Result<ScanRes> {
    let tagged_file = read_from_path(path).wrap_err("Failed to read file")?;
    let tag = tagged_file
        .primary_tag()
        .cloned()
        .unwrap_or_else(|| Tag::new(tagged_file.primary_tag_type()));

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

    info!(
        "Best match release/recording was '{}({})' / '{}({})' with score {}",
        best_release.release_group.title,
        best_release.id,
        best_recording.title,
        best_recording.id,
        best_score
    );

    let mb_client = MusicbrainzClient::new();
    let release_additional_data = mb_client.release(&best_release.id).await?;

    let old_metadata = Metadata::from_tag(&tag);

    let new_metadata = format_to_metadata(best_recording, best_release, release_additional_data)?;

    Ok(ScanRes {
        old_metadata,
        new_metadata,
        old_tag: tag,
        scanner_info: scanner_res.log,
    })
}
