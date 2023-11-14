use std::path::Path;

use anyhow::Context;
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

    let release_selector = &CONFIG.read().release_selector.clone();
    let mb_client = MusicbrainzClient::new();
    let tasks = best_match.recordings.iter().map(|id| async {
        let res = mb_client.recording(&id.id).await;
        if let Ok(recording) = res {
            let best_release = recording.releases.into_iter().max_by(|a, b| {
                let score_a = calc_release_score(a, release_selector);
                let score_b = calc_release_score(b, release_selector);
                score_a.partial_cmp(&score_b).unwrap()
            });
            match best_release {
                Some(best_release) => Some(RecordingMetadata {
                    title: recording.title,
                    id: recording.id,
                    release: best_release,
                }),
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
    let best = best_release_per_recording
        .into_iter()
        .flatten()
        .max_by(|a, b| {
            calc_release_score(&a.release, release_selector)
                .partial_cmp(&calc_release_score(&b.release, release_selector))
                .unwrap()
        })
        .context("No best release/recording found.")?;
    info!(
        "Best match release/recording was '{}({})' / '{}({})'",
        best.release.release_group.title, best.release.id, best.title, best.id
    );

    Ok(())
}
