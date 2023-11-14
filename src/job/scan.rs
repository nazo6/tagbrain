use std::path::Path;

use anyhow::Context;
use serde::Deserialize;

use crate::{
    api::{acoustid::AcoustidClient, musicbrainz::MusicbrainzClient},
    config::CONFIG,
};

#[tracing::instrument(err)]
pub(super) async fn scan_and_move(path: &Path) -> anyhow::Result<()> {
    let calculated = calc_fingerprint(path).await?;
    let acoustid_client = AcoustidClient::new();
    let acoustid_res = acoustid_client
        .lookup(&calculated.fingerprint, calculated.duration.round() as i64)
        .await?;
    let best_match = acoustid_res
        .results
        .iter()
        .max_by(|a, b| a.score.partial_cmp(&b.score).unwrap());
    if let Some(best_match) = best_match {
        if best_match.score >= CONFIG.read().acoustid_match_threshold {
            let mb_client = MusicbrainzClient::new();
            let tasks = best_match
                .recordings
                .iter()
                .map(|id| async {
                    let res = mb_client.recording(&id.id).await;
                    dbg!(&res);
                    res
                })
                .collect::<Vec<_>>();
            let results = futures::future::join_all(tasks).await;
            let results = results
                .into_iter()
                .filter_map(|res| res.ok())
                .collect::<Vec<_>>();
        }
    }
    Ok(())
}

#[derive(Deserialize, Debug)]
struct FpcalcResult {
    duration: f64,
    fingerprint: String,
}
#[tracing::instrument]
async fn calc_fingerprint(path: &Path) -> anyhow::Result<FpcalcResult> {
    let output = tokio::process::Command::new("fpcalc")
        .arg(path)
        .arg("-json")
        .output()
        .await
        .context("Failed to run fpcalc")?;
    let str = String::from_utf8(output.stdout)?;
    let json: FpcalcResult = serde_json::from_str(&str)?;

    Ok(json)
}
