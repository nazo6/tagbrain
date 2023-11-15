use std::{
    path::{Path, PathBuf},
    str::FromStr,
};

use anyhow::Context;
use lofty::{Accessor, Tag};
use serde::Deserialize;
use tokio::io::AsyncWriteExt;

use crate::{
    api::musicbrainz::recording::RecordingResRelease,
    config::{ReleaseSelector, CONFIG},
};

use super::metadata::Metadata;

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

macro_rules! write_property {
    ($title:literal, $old:ident, $new:ident, $prop:ident, $text:ident) => {
        $text.push_str(&format!(
            "{}\t: {} → {}\n",
            $title,
            $old.$prop.as_ref().unwrap_or(&"NULL".to_string()),
            $new.$prop.as_ref().unwrap_or(&"NULL".to_string())
        ));
    };
}
macro_rules! write_property_num {
    ($title:literal, $old:ident, $new:ident, $prop:ident, $text:ident) => {
        $text.push_str(&format!(
            "{}:\t {} → {}\n",
            $title,
            $old.$prop
                .as_ref()
                .map(|p| p.to_string())
                .unwrap_or("NULL".to_string()),
            $new.$prop
                .as_ref()
                .map(|p| p.to_string())
                .unwrap_or("NULL".to_string())
        ));
    };
}

pub(super) async fn log_diff(
    old_path: &Path,
    old: &Metadata,
    new: &Metadata,
) -> anyhow::Result<()> {
    let mut log_path = PathBuf::from_str(&CONFIG.read().log_dir)?;
    log_path.push(old_path.file_name().context("No file name")?);

    let mut text = "----------------\n".to_string();
    write_property!("Title", old, new, title, text);
    write_property!("Artist", old, new, artist, text);
    write_property!("Artist Sort", old, new, artist_sort, text);
    write_property!("Album", old, new, album, text);
    write_property!("Album Artist", old, new, album_artist, text);
    write_property!("Album Artist Sort", old, new, album_artist_sort, text);
    write_property_num!("Track", old, new, track, text);
    write_property_num!("Total Tracks", old, new, total_tracks, text);
    write_property_num!("Disk", old, new, disk, text);
    write_property_num!("Total Disks", old, new, total_disks, text);
    write_property!("Date", old, new, date, text);
    write_property!("Year", old, new, year, text);
    write_property!("Label", old, new, label, text);
    write_property!("Media", old, new, media, text);
    write_property!("Musicbrainz Track ID", old, new, musicbrainz_track_id, text);
    write_property!("Musicbrainz Album ID", old, new, musicbrainz_album_id, text);
    write_property!(
        "Musicbrainz Artist ID",
        old,
        new,
        musicbrainz_artist_id,
        text
    );
    text.push_str("----------------\n\n");

    let mut log_file = tokio::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_path)
        .await?;
    log_file.write_all(text.as_bytes()).await?;

    Ok(())
}
