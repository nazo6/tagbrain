use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use lofty::{read_from_path, Tag, TaggedFileExt};
use tracing::{info, warn};

use crate::{
    api::{
        acoustid::{AcoustidClient, LookupResEntry},
        musicbrainz::{
            recording::{RecordingRes, RecordingResRelease},
            release::ReleaseRes,
            ArtistCreditVecToString, MusicbrainzClient,
        },
    },
    config::CONFIG,
};

use self::metadata::Metadata;
use metadata::write_metadata;
use utils::{calc_fingerprint, calc_release_score};

mod metadata;
mod utils;

#[tracing::instrument()]
async fn acoustid_find(fingerprint: &str, duration: f64) -> Result<Option<LookupResEntry>> {
    let acoustid_client = AcoustidClient::new();
    let acoustid_res = acoustid_client
        .lookup(fingerprint, duration.round() as u32)
        .await?;
    let Some(best) = acoustid_res
        .results
        .into_iter()
        .max_by(|a, b| a.score.partial_cmp(&b.score).unwrap()) else {
            return Ok(None);
        };

    if best.score < CONFIG.read().acoustid_match_threshold {
        return Ok(None);
    }

    Ok(Some(best))
}

fn find_best_release_and_recording(
    recordings: Vec<RecordingRes>,
    crr_tag: Option<&Tag>,
) -> Option<(RecordingRes, RecordingResRelease, f64)> {
    let release_selector = &CONFIG.read().release_selector.clone();
    let best_recording_releases: Vec<(RecordingRes, RecordingResRelease, f64)> = recordings
        .into_iter()
        .map(|recording| {
            let (best_release, best_score) =
                recording
                    .releases
                    .iter()
                    .fold((None, -1.0), |(best, best_score), release| {
                        let score = calc_release_score(release, crr_tag, release_selector);
                        if score > best_score {
                            (Some(release), score)
                        } else {
                            (best, best_score)
                        }
                    });
            let best_release = best_release.cloned();
            match best_release {
                Some(best_release) => Some((recording, best_release, best_score)),
                None => {
                    warn!(
                        "No release found for recording: {} ({})",
                        recording.title, recording.id
                    );
                    None
                }
            }
        })
        .collect::<Vec<_>>()
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();
    best_recording_releases
        .into_iter()
        .max_by(|a, b| a.2.partial_cmp(&b.2).expect("This should not happen."))
}

/// Collect data, and format it into a metadata struct.
fn format_to_metadata(
    recording: RecordingRes,
    release: RecordingResRelease,
    release_additional: ReleaseRes,
) -> Result<Metadata> {
    let this_media = release_additional
        .media
        .iter()
        .find(|media| {
            media
                .tracks
                .iter()
                .any(|track| track.recording.id == recording.id)
        })
        .context("No media found! This should not happen!")?;
    let this_track = this_media
        .tracks
        .iter()
        .find(|track| track.recording.id == recording.id)
        .expect("recording id not found. this must be unreachable!");

    let metadata = Metadata {
        title: Some(recording.title),
        artist: Some(recording.artist_credit.to_string()),
        artist_sort: Some(recording.artist_credit.to_sort_string()),
        album: Some(release_additional.title),
        album_artist: Some(release_additional.artist_credit.to_string()),
        album_artist_sort: Some(release_additional.artist_credit.to_sort_string()),
        track: Some(this_track.position.to_string()),
        total_tracks: Some(this_media.track_count.to_string()),
        disk: Some(this_media.position.to_string()),
        total_disks: Some(release_additional.media.len().to_string()),
        year: recording
            .first_release_date
            .split('-')
            .next()
            .map(|s| s.to_owned()),
        date: Some(recording.first_release_date),
        label: release_additional.label_info.and_then(|label| {
            label
                .first()
                .and_then(|li| li.label.as_ref().map(|label| label.id.clone()))
        }),
        media: release.release_group.primary_type,
        musicbrainz_track_id: Some(this_track.id.clone()),
        musicbrainz_album_id: Some(release.id),
        musicbrainz_artist_id: recording
            .artist_credit
            .first()
            .map(|ac| ac.artist.id.clone()),
        musicbrainz_release_artist_id: release_additional
            .artist_credit
            .first()
            .map(|ac| ac.artist.id.clone()),
        musicbrainz_release_group_id: Some(release.release_group.id),
    };

    Ok(metadata)
}

pub struct ScanResult {
    pub old_metadata: Metadata,
    pub new_metadata: Metadata,
    pub acoustid_score: f64,
}

#[tracing::instrument(err)]
pub(super) async fn scan_and_copy(path: &Path) -> anyhow::Result<ScanResult> {
    info!("Scanning file: {}", path.display());
    let calculated = calc_fingerprint(path).await?;
    let acoustid_match = acoustid_find(&calculated.fingerprint, calculated.duration)
        .await?
        .context("No acoustid match found.")?;
    info!(
        "Best match acoustid was {} (score: {})",
        acoustid_match.id, acoustid_match.score
    );

    let tagged_file = read_from_path(path).ok();
    let tag = tagged_file.as_ref().and_then(|f| f.primary_tag());

    let mb_client = MusicbrainzClient::new();

    let (best_recording, best_release, best_score) = {
        let recordings =
            futures::future::join_all(acoustid_match.recordings.iter().flatten().map(|id| async {
                let res = mb_client.recording(&id.id).await;
                if let Err(e) = &res {
                    warn!("Failed to get recording {}: {}", id.id, e);
                }
                res
            }))
            .await
            .into_iter()
            .filter_map(|res| res.ok())
            .collect::<Vec<_>>();

        if recordings.is_empty() {
            return Err(anyhow::anyhow!("No recordings found!"));
        }

        find_best_release_and_recording(recordings, tag).context("Failed to find best match")?
    };

    info!(
        "Best match release/recording was '{}({})' / '{}({})' with score {}",
        best_release.release_group.title,
        best_release.id,
        best_recording.title,
        best_recording.id,
        best_score
    );

    let release_additional_data = mb_client.release(&best_release.id).await?;
    let metadata = format_to_metadata(best_recording, best_release, release_additional_data)?;

    let Some(Some(ext)) = path.extension().map(|ext| ext.to_str()) else {
        return Err(anyhow::anyhow!("No extension found!"));
    };
    let mut new_path = PathBuf::new();
    new_path.push(CONFIG.read().target_dir.clone());
    new_path.push(
        metadata
            .album_artist
            .clone()
            .expect("album artist not found"),
    );
    new_path.push(metadata.album.clone().expect("album not found"));
    new_path.push(format!(
        "{} - {}.{}",
        metadata.track.as_ref().expect("track not found"),
        metadata.title.as_ref().expect("title not found"),
        ext
    ));

    tokio::fs::create_dir_all(new_path.parent().unwrap()).await?;
    tokio::fs::copy(path, &new_path).await?;

    let mut tagged_file_new = read_from_path(&new_path)
        .ok()
        .context("Failed to open new file")?;
    let tag_new = tagged_file_new
        .primary_tag_mut()
        .context("Failed to open new file")?;
    write_metadata(tag_new, metadata.clone());

    let old_metadata = if let Some(tag) = tag {
        Metadata::from_tag(tag)
    } else {
        Metadata::default()
    };

    Ok(ScanResult {
        old_metadata,
        new_metadata: metadata,
        acoustid_score: acoustid_match.score,
    })
}
