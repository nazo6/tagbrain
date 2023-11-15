use std::path::Path;

use anyhow::Context;
use lofty::{Accessor, Tag};
use serde::Deserialize;

use crate::{api::musicbrainz::recording::RecordingResRelease, config::ReleaseSelector};

#[derive(Deserialize, Debug)]
pub(super) struct FpcalcResult {
    pub duration: f64,
    pub fingerprint: String,
}
pub(super) async fn calc_fingerprint(path: &Path) -> anyhow::Result<FpcalcResult> {
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

pub(super) fn calc_release_score(
    release: &RecordingResRelease,
    current_tag: Option<&Tag>,
    release_selector: &ReleaseSelector,
) -> f64 {
    let mut score = 0.0;
    if let Some(country) = &release.country {
        let idx = release_selector
            .country
            .preferred
            .iter()
            .position(|item| item == &country.to_uppercase());
        if let Some(idx) = idx {
            score += release_selector.country.weight * (1.0 / (idx as f64 + 1.0))
        }
    }
    if let Some(primary_type) = &release.release_group.primary_type {
        let idx = release_selector
            .release_group_type
            .preferred
            .iter()
            .position(|item| item == &primary_type.to_lowercase());
        if let Some(idx) = idx {
            score += release_selector.release_group_type.weight * (1.0 / (idx as f64 + 1.0))
        }
    }

    let title_distance_score = if let Some(Some(album)) = current_tag.map(|t| t.album()) {
        let distance_score = strsim::normalized_levenshtein(&album, &release.release_group.title);
        if distance_score >= release_selector.release_title_distance.threshold {
            distance_score
        } else {
            0.0
        }
    } else {
        0.0
    };
    score += release_selector.release_title_distance.weight * title_distance_score;

    score
}
