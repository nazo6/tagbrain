use std::path::Path;

use eyre::{eyre, Context, Result};
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
    interface::metadata::Metadata,
};

use super::utils::{calc_fingerprint, calc_release_score};

pub(super) struct ScanRes {
    pub old_metadata: Metadata,
    pub new_metadata: Metadata,
    pub old_tag: Tag,
    pub acoustid_score: f64,
}
pub(super) async fn scan(path: &Path) -> eyre::Result<ScanRes> {
    info!("Scanning file: {}", path.display());
    let calculated = calc_fingerprint(path)
        .await
        .wrap_err("Failed to calc fingerprint")?;
    let acoustid_match = acoustid_find(&calculated.fingerprint, calculated.duration).await?;
    debug!(
        "Best match acoustid was {} (score: {})",
        acoustid_match.id, acoustid_match.score
    );

    let tagged_file = read_from_path(path).wrap_err("Failed to read file")?;
    let tag = tagged_file
        .primary_tag()
        .cloned()
        .unwrap_or_else(|| Tag::new(tagged_file.primary_tag_type()));

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

        find_best_release_and_recording(recordings, &tag)
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

    let old_metadata = Metadata::from_tag(&tag);

    let new_metadata = format_to_metadata(best_recording, best_release, release_additional_data)?;

    Ok(ScanRes {
        old_metadata,
        new_metadata,
        old_tag: tag,
        acoustid_score: acoustid_match.score,
    })
}

#[tracing::instrument]
async fn acoustid_find(fingerprint: &str, duration: f64) -> Result<LookupResEntry, eyre::Report> {
    let acoustid_client = AcoustidClient::new();
    let acoustid_res = acoustid_client
        .lookup(fingerprint, duration.round() as u32)
        .await?;
    let Some(best) = acoustid_res
        .results
        .into_iter()
        .max_by(|a, b| a.score.partial_cmp(&b.score).unwrap())
    else {
        return Err(eyre!("No acoustid match found."));
    };

    if best.score < CONFIG.read().acoustid_match_threshold {
        return Err(eyre!(
            "Best acoustid match score is too low. Score: {}",
            best.score
        ));
    }

    Ok(best)
}

fn find_best_release_and_recording(
    recordings: Vec<RecordingRes>,
    crr_tag: &Tag,
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
        original_date: release.release_group.first_release_date,
        date: release.date.clone(),
        year: release
            .date
            .and_then(|d| d.split('-').next().map(|s| s.to_owned())),
        label: release_additional.label_info.and_then(|label| {
            label
                .first()
                .and_then(|li| li.label.as_ref().map(|label| label.name.clone()))
        }),
        media: Some(this_media.format.clone()),
        script: release_additional
            .text_representation
            .and_then(|tr| tr.script),
        musicbrainz_artist_id: recording
            .artist_credit
            .and_then(|ac| ac.first().map(|ac| ac.artist.id.clone())),
        musicbrainz_track_id: Some(this_track.id.clone()),
        musicbrainz_release_id: Some(release.id),
        musicbrainz_release_artist_id: release_additional
            .artist_credit
            .and_then(|ac| ac.first().map(|ac| ac.artist.id.clone())),
        musicbrainz_release_group_id: Some(release.release_group.id),
        musicbrainz_recording_id: Some(recording.id),
    };

    Ok(metadata)
}
