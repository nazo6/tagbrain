use lofty::{Accessor, Tag};
use tracing::warn;

use crate::{
    api::musicbrainz::recording::{RecordingRes, RecordingResRelease},
    config::{ReleaseSelector, CONFIG},
};

pub(super) fn calc_score(
    release: &RecordingResRelease,
    recording: &RecordingRes,
    current_tag: &Tag,
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

    let release_title_distance_score = if let Some(album) = current_tag.album() {
        let distance_score = strsim::normalized_levenshtein(&album, &release.release_group.title);
        if distance_score >= release_selector.release_title_distance.threshold {
            distance_score
        } else {
            0.0
        }
    } else {
        0.0
    };
    score += release_selector.release_title_distance.weight * release_title_distance_score;

    let recording_title_distance_score = if let Some(album) = current_tag.title() {
        let distance_score = strsim::normalized_levenshtein(&album, &recording.title);
        if distance_score >= release_selector.recording_title_distance.threshold {
            distance_score
        } else {
            0.0
        }
    } else {
        0.0
    };
    score += release_selector.recording_title_distance.weight * recording_title_distance_score;

    score
}

pub(super) fn find_best_release_and_recording(
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
                        let score = calc_score(release, &recording, crr_tag, release_selector);
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
