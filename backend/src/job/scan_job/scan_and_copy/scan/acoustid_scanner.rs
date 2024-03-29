use std::path::Path;

use eyre::{eyre, Result};
use tracing::{debug, info, warn};

use crate::{
    api::{acoustid::AcoustidClient, musicbrainz::MusicbrainzClient},
    config::CONFIG,
    job::scan_job::scan_and_copy::{scan::ScannerRes, ScannerInfo},
};

use super::FpcalcResult;

#[tracing::instrument(skip(fp))]
pub(super) async fn acoustid_scanner(
    path: &Path,
    fp: &FpcalcResult,
) -> Result<ScannerRes, eyre::Report> {
    info!("Scanning file: {}", path.display());

    let acoustid_client = AcoustidClient::new();
    let acoustid_res = acoustid_client
        .lookup(&fp.fingerprint, fp.duration.round() as u32)
        .await?;
    let Some(best) = acoustid_res
        .results
        .into_iter()
        .max_by(|a, b| a.score.partial_cmp(&b.score).unwrap())
    else {
        return Err(eyre!("No acoustid match found."));
    };

    let Some(best_recordings) = best.recordings else {
        return Err(eyre!("No acoustid match found."));
    };

    if best.score < CONFIG.read().acoustid_match_threshold {
        return Err(eyre!(
            "Best acoustid match score is too low. Score: {}",
            best.score
        ));
    }

    debug!(
        "Best match acoustid was {} (score: {})",
        best.id, best.score
    );

    let recordings = futures::future::join_all(best_recordings.iter().map(|id| async move {
        let mb_client = MusicbrainzClient::new();
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

    Ok(ScannerRes {
        log: ScannerInfo::AcoustId { score: best.score },
        recordings,
    })
}
