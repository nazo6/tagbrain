use std::path::{Path, PathBuf};

use crate::{
    api::musicbrainz::{recording::RecordingRes, release::ReleaseRes, ArtistCreditVecToString},
    config::CONFIG,
    interface::metadata::Metadata,
};
use eyre::{eyre, Context, Result};
use lofty::{read_from_path, Picture, Tag, TaggedFileExt};
use sanitize_filename::sanitize;

/// Collect data, and format it into a metadata struct.
pub(super) fn response_to_metadata(
    recording: RecordingRes,
    release: ReleaseRes,
) -> Result<Metadata> {
    let this_media = release
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
        album: Some({
            // let mut album = release.title.clone();
            // if let Some(disambiguation) = release.disambiguation {
            //     if !disambiguation.is_empty() {
            //         album.push_str(" (");
            //         album.push_str(&disambiguation);
            //         album.push(')');
            //     }
            // }
            // album
            release.title.clone()
        }),
        album_artist: release.artist_credit.as_ref().map(|a| a.to_string()),
        album_artist_sort: release.artist_credit.as_ref().map(|a| a.to_sort_string()),
        track: Some(this_track.position.to_string()),
        total_tracks: Some(this_media.track_count.to_string()),
        disk: Some(this_media.position.to_string()),
        total_disks: Some(release.media.len().to_string()),
        original_date: release.release_group.first_release_date,
        date: release.date.clone(),
        year: release
            .date
            .and_then(|d| d.split('-').next().map(|s| s.to_owned())),
        label: release.label_info.and_then(|label| {
            label
                .first()
                .and_then(|li| li.label.as_ref().map(|label| label.name.clone()))
        }),
        media: Some(this_media.format.clone()),
        script: release.text_representation.and_then(|tr| tr.script),
        musicbrainz_artist_id: recording
            .artist_credit
            .and_then(|ac| ac.first().map(|ac| ac.artist.id.clone())),
        musicbrainz_track_id: Some(this_track.id.clone()),
        musicbrainz_release_id: Some(release.id),
        musicbrainz_release_artist_id: release
            .artist_credit
            .and_then(|ac| ac.first().map(|ac| ac.artist.id.clone())),
        musicbrainz_release_group_id: Some(release.release_group.id),
        musicbrainz_recording_id: Some(recording.id),
    };

    Ok(metadata)
}

#[tracing::instrument]
pub(super) async fn fetch_cover_art(release_id: &str) -> eyre::Result<Picture> {
    let cover_art = reqwest::get(&format!(
        "https://coverartarchive.org/release/{}/front",
        release_id
    ))
    .await?
    .bytes()
    .await?;
    let mut cover_art = std::io::Cursor::new(cover_art);
    let picture = Picture::from_reader(&mut cover_art)?;
    Ok(picture)
}

/// Read tag from file. If file has no tag, return default tag.
pub(super) fn read_tag_or_default(path: &Path) -> eyre::Result<Tag> {
    let tagged_file = read_from_path(path).wrap_err("Failed to read file")?;
    let tag = tagged_file
        .primary_tag()
        .cloned()
        .unwrap_or_else(|| Tag::new(tagged_file.primary_tag_type()));
    Ok(tag)
}

/// Determine the save path from metadata and source path (for detecting extension).
pub(super) fn get_save_path_from_metadata(
    source_path: &Path,
    metadata: &Metadata,
) -> eyre::Result<PathBuf> {
    let Some(Some(ext)) = source_path.extension().map(|ext| ext.to_str()) else {
        return Err(eyre!("No extension found!"));
    };

    let mut new_path = PathBuf::new();

    let artist = sanitize(
        metadata
            .artist
            .as_ref()
            .ok_or_else(|| eyre!("artist not found"))?,
    );
    let album = sanitize(
        metadata
            .album
            .as_ref()
            .ok_or_else(|| eyre!("album not found"))?,
    );
    let title = sanitize(
        metadata
            .title
            .as_ref()
            .ok_or_else(|| eyre!("title not found"))?,
    );
    let album_artist = &metadata.album_artist;
    let track = &metadata.track;

    new_path.push(CONFIG.read().target_dir.clone());
    new_path.push(sanitize(album_artist.clone().unwrap_or(artist.clone())));
    new_path.push(album.clone());
    let file_name = {
        let mut file_name = String::new();
        if let Some(track) = track {
            file_name.push_str(&sanitize(track));
            file_name.push_str(" - ");
        }
        file_name.push_str(&title);
        file_name.push('.');
        file_name.push_str(ext);
        file_name
    };
    new_path.push(file_name);
    Ok(new_path)
}

#[cfg(test)]
mod test {
    #[tokio::test]
    async fn cover_art() {
        let cover_art = super::fetch_cover_art("db85c244-53e7-441c-bab0-52c9c0d27450")
            .await
            .unwrap();
        assert_eq!(cover_art.mime_type().to_string(), "image/jpeg".to_string());
    }
}
