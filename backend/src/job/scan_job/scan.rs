use std::path::{Path, PathBuf};

use eyre::{eyre, Result};
use lofty::{read_from_path, Tag, TaggedFileExt};
use tracing::{debug, info, warn};

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
    interface::metadata::write_metadata,
};

use utils::{calc_fingerprint, calc_release_score};

use crate::interface::metadata::Metadata;

mod utils;

#[tracing::instrument]
async fn acoustid_find(fingerprint: &str, duration: f64) -> Result<Option<LookupResEntry>> {
    let acoustid_client = AcoustidClient::new();
    let acoustid_res = acoustid_client
        .lookup(fingerprint, duration.round() as u32)
        .await?;
    let Some(best) = acoustid_res
        .results
        .into_iter()
        .max_by(|a, b| a.score.partial_cmp(&b.score).unwrap())
    else {
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
        .ok_or_else(|| eyre::eyre!("No media found"))?;
    let this_track = this_media
        .tracks
        .iter()
        .find(|track| track.recording.id == recording.id)
        .expect("recording id not found. this must be unreachable!");

    let metadata = Metadata {
        title: Some(recording.title),
        artist: recording.artist_credit.as_ref().map(|a| a.to_string()),
        artist_sort: recording.artist_credit.as_ref().map(|a| a.to_sort_string()),
        album: Some(release_additional.title),
        album_artist: release_additional
            .artist_credit
            .as_ref()
            .map(|a| a.to_string()),
        album_artist_sort: release_additional
            .artist_credit
            .as_ref()
            .map(|a| a.to_sort_string()),
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
                .and_then(|li| li.label.as_ref().map(|label| label.name.clone()))
        }),
        media: release.release_group.primary_type,
        musicbrainz_track_id: Some(this_track.id.clone()),
        musicbrainz_album_id: Some(release.id),
        musicbrainz_artist_id: recording
            .artist_credit
            .and_then(|ac| ac.first().map(|ac| ac.artist.id.clone())),
        musicbrainz_release_artist_id: release_additional
            .artist_credit
            .and_then(|ac| ac.first().map(|ac| ac.artist.id.clone())),
        musicbrainz_release_group_id: Some(release.release_group.id),
    };

    Ok(metadata)
}

pub struct ScanSuccessLog {
    pub old_metadata: Metadata,
    pub new_metadata: Metadata,
    pub acoustid_score: f64,
    pub target_path: PathBuf,
}

#[tracing::instrument]
pub(super) async fn scan_and_copy(path: &Path) -> eyre::Result<ScanSuccessLog> {
    info!("Scanning file: {}", path.display());
    let calculated = calc_fingerprint(path).await?;
    let acoustid_match = acoustid_find(&calculated.fingerprint, calculated.duration)
        .await?
        .ok_or_else(|| eyre!("No acoustid match found."))?;
    debug!(
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
                    warn!("Failed to get recording {}: {:?}", id.id, e);
                }
                res
            }))
            .await
            .into_iter()
            .filter_map(|res| res.ok())
            .collect::<Vec<_>>();

        find_best_release_and_recording(recordings, tag)
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

    let release_additional_data = mb_client.release(&best_release.id).await?;
    let metadata = format_to_metadata(best_recording, best_release, release_additional_data)?;

    let Some(Some(ext)) = path.extension().map(|ext| ext.to_str()) else {
        return Err(eyre!("No extension found!"));
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

    if let Ok(exist) = tokio::fs::try_exists(&new_path).await {
        if exist && !CONFIG.read().overwrite {
            return Err(eyre!("File already exists! Skipping..."));
        }
    }
    tokio::fs::create_dir_all(new_path.parent().unwrap()).await?;
    tokio::fs::copy(path, &new_path).await?;
    if CONFIG.read().delete_original {
        let res = tokio::fs::remove_file(path).await;
        if let Err(e) = res {
            warn!("Failed to delete original file: {}", e);
        }
        let source_dir = CONFIG.read().source_dir.clone();
        if let (Ok(source_dir_abs), Ok(original_path_abs)) =
            (Path::new(&source_dir).canonicalize(), path.canonicalize())
        {
            let res = tokio::task::spawn_blocking(move || {
                delete_empty_folder_recursive(&original_path_abs, &source_dir_abs)
            })
            .await;
            if let Err(e) = res {
                warn!("Failed to delete empty folder: {}", e);
            }
        }
    }

    let mut tagged_file_new = read_from_path(&new_path)
        .ok()
        .ok_or_else(|| eyre!("Failed to open new file"))?;
    let tag_new = tagged_file_new
        .primary_tag_mut()
        .ok_or_else(|| eyre!("Failed to open new file"))?;
    write_metadata(tag_new, metadata.clone());

    let old_metadata = if let Some(tag) = tag {
        Metadata::from_tag(tag)
    } else {
        Metadata::default()
    };

    Ok(ScanSuccessLog {
        old_metadata,
        new_metadata: metadata,
        acoustid_score: acoustid_match.score,
        target_path: new_path,
    })
}

fn delete_empty_folder_recursive(path: &Path, stop_at: &Path) -> Result<()> {
    if path == stop_at {
        return Ok(());
    }
    if path.is_dir() {
        let mut is_empty = true;
        for entry in std::fs::read_dir(path)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                delete_empty_folder_recursive(&path, stop_at)?;
            }
            is_empty = false;
        }
        if is_empty {
            std::fs::remove_dir(path)?;
        }
    }
    Ok(())
}
