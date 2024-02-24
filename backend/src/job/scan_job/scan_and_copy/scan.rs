use std::path::Path;

use eyre::{eyre, Context};
use lofty::Tag;
use serde::Deserialize;
use tracing::{info, warn};

use crate::{
    api::{
        acoustid::AcoustidClient,
        musicbrainz::{recording::RecordingRes, MusicbrainzClient},
    },
    config::CONFIG,
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

#[derive(Deserialize, Debug)]
pub(super) struct FpcalcResult {
    pub duration: f64,
    pub fingerprint: String,
}
async fn calc_fingerprint(path: &Path) -> eyre::Result<FpcalcResult> {
    let output = tokio::process::Command::new("fpcalc")
        .arg(path)
        .arg("-json")
        .output()
        .await
        .wrap_err("Failed to run fpcalc")?;
    let str = String::from_utf8(output.stdout)?;
    let json: FpcalcResult = serde_json::from_str(&str)?;

    Ok(json)
}

pub(super) async fn scan(path: &Path) -> eyre::Result<ScanRes> {
    let mut tag = read_tag_or_default(path)?;

    let old_metadata = Metadata::from_tag(&tag);

    let fp = calc_fingerprint(path)
        .await
        .wrap_err("Failed to calc fingerprint")?;

    if old_metadata.musicbrainz_release_id.is_some()
        && old_metadata.musicbrainz_recording_id.is_some()
        && CONFIG.read().force
    {
        return Ok(ScanRes {
            old_metadata: old_metadata.clone(),
            new_metadata: old_metadata,
            new_tag: tag,
            scanner_info: ScannerInfo::Skip,
        });
    }

    let (scanner_res, submit_fingerprint) =
        if let Ok(res) = acoustid_scanner::acoustid_scanner(path, &fp).await {
            (res, false)
        } else {
            info!("Acoustid scanner failed. Falling back to musicbrainz search scanner.");
            (
                musicbrainz_search_scanner::musicbrainz_search_scanner(&tag).await?,
                true,
            )
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

    if submit_fingerprint {
        let acoustid_client = AcoustidClient::new();
        let _ = acoustid_client
            .submit(
                &best_recording.id,
                &fp.fingerprint,
                fp.duration.round() as u32,
            )
            .await;
        info!("Submitted fingerprint to acoustid: {}", best_recording.id);
    }

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
