use std::path::Path;

use anyhow::{anyhow, Context};
use lofty::{read_from_path, TaggedFileExt};
use tracing::{info, warn};

use crate::{
    api::{
        acoustid::AcoustidClient,
        musicbrainz::{MusicbrainzClient, RecordingResRelease},
    },
    config::CONFIG,
    job::scan::utils::{calc_fingerprint, calc_release_score},
};

mod utils;

#[derive(Debug)]
struct RecordingMetadata {
    title: String,
    id: String,
    release: RecordingResRelease,
}

#[tracing::instrument(err)]
pub(super) async fn scan_and_move(path: &Path) -> anyhow::Result<()> {
    info!("Scanning file: {}", path.display());
    let calculated = calc_fingerprint(path).await?;
    let acoustid_client = AcoustidClient::new();
    let acoustid_res = acoustid_client
        .lookup(&calculated.fingerprint, calculated.duration.round() as i64)
        .await?;
    let best_match = acoustid_res
        .results
        .iter()
        .max_by(|a, b| a.score.partial_cmp(&b.score).unwrap())
        .context("No acoustid match found.")?;
    info!(
        "Best match acoustid was {} (score: {})",
        best_match.id, best_match.score
    );

    if best_match.score < CONFIG.read().acoustid_match_threshold {
        return Err(anyhow::anyhow!(
            "Best match score was too low ({}). Skipped",
            best_match.score
        ));
    }

    let tagged_file = read_from_path(path).ok();
    let tag = tagged_file.as_ref().and_then(|f| f.primary_tag());

    let release_selector = &CONFIG.read().release_selector.clone();
    let mb_client = MusicbrainzClient::new();
    let tasks = best_match.recordings.iter().map(|id| async {
        let res = mb_client.recording(&id.id).await;
        if let Ok(recording) = res {
            let (best_release, best_score) =
                recording
                    .releases
                    .into_iter()
                    .fold((None, 0.0), |(best, best_score), release| {
                        let score = calc_release_score(&release, tag, release_selector);
                        if score > best_score {
                            (Some(release), score)
                        } else {
                            (best, best_score)
                        }
                    });
            match best_release {
                Some(best_release) => Some((
                    RecordingMetadata {
                        title: recording.title,
                        id: recording.id,
                        release: best_release,
                    },
                    best_score,
                )),
                None => {
                    warn!(
                        "No release found for recording: {} ({})",
                        recording.title, recording.id
                    );
                    None
                }
            }
        } else {
            None
        }
    });
    let best_release_per_recording = futures::future::join_all(tasks).await;
    let (best, best_score) = best_release_per_recording.into_iter().flatten().fold(
        (None, 0.0),
        |(best, best_score), (crr_data, crr_score)| {
            if crr_score > best_score {
                (Some(crr_data), crr_score)
            } else {
                (best, crr_score)
            }
        },
    );
    let Some(best) = best else {
        return Err(anyhow!("No best match found."));
    };
    info!(
        "Best match release/recording was '{}({})' / '{}({})' with score {}",
        best.release.release_group.title, best.release.id, best.title, best.id, best_score
    );

    Ok(())
}
