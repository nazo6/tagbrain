use std::path::Path;

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
    job::scan::utils::{calc_fingerprint, calc_release_score},
};

mod utils;

#[derive(Debug)]
pub struct Metadata {
    pub title: String,
    pub artist: String,
    pub artist_sort: Option<String>,
    pub album: String,
    pub album_artist: String,
    pub album_artist_sort: Option<String>,
    pub track: i64,
    pub total_tracks: Option<i64>,
    pub disc: i64,
    pub total_discs: Option<i64>,
    pub date: Option<String>,
    pub year: Option<String>,
    pub label: Option<String>,
    pub media: Option<String>,
    pub musicbrainz_track_id: Option<String>,
    pub musicbrainz_album_id: Option<String>,
    pub musicbrainz_artist_id: Option<String>,
    pub musicbrainz_album_artist_id: Option<String>,
    pub musicbrainz_release_group_id: Option<String>,
}

async fn acoustid_find(fingerprint: &str, duration: f64) -> Result<Option<LookupResEntry>> {
    let acoustid_client = AcoustidClient::new();
    let acoustid_res = acoustid_client
        .lookup(fingerprint, duration.round() as i64)
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
                    .fold((None, 0.0), |(best, best_score), release| {
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
        title: recording.title,
        artist: recording.artist_credit.to_string(),
        artist_sort: Some(recording.artist_credit.to_sort_string()),
        album: release_additional.title,
        album_artist: release_additional.artist_credit.to_string(),
        album_artist_sort: Some(release_additional.artist_credit.to_sort_string()),
        track: this_track.position,
        total_tracks: Some(this_media.track_count),
        disc: this_media.position,
        total_discs: Some(release_additional.media.len() as i64),
        year: recording
            .first_release_date
            .split('-')
            .next()
            .map(|s| s.to_owned()),
        date: Some(recording.first_release_date),
        label: release_additional
            .label_info
            .first()
            .map(|li| li.label.name.clone()),
        media: Some(release.release_group.primary_type),
        musicbrainz_track_id: Some(this_track.id.clone()),
        musicbrainz_album_id: Some(release.id),
        musicbrainz_artist_id: recording
            .artist_credit
            .first()
            .map(|ac| ac.artist.id.clone()),
        musicbrainz_album_artist_id: release_additional
            .artist_credit
            .first()
            .map(|ac| ac.artist.id.clone()),
        musicbrainz_release_group_id: Some(release.release_group.id),
    };

    Ok(metadata)
}

#[tracing::instrument(err)]
pub(super) async fn scan_and_move(path: &Path) -> anyhow::Result<()> {
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
        let recordings = futures::future::join_all(
            acoustid_match
                .recordings
                .iter()
                .map(|id| async { mb_client.recording(&id.id).await }),
        )
        .await
        .into_iter()
        .filter_map(|res| res.ok())
        .collect::<Vec<_>>();
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

    dbg!(&metadata);

    Ok(())
}
