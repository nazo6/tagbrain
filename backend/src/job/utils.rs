use std::path::{Path, PathBuf};

use crate::{
    api::musicbrainz::{recording::RecordingRes, release::ReleaseRes, ArtistCreditVecToString},
    interface::metadata::Metadata,
};
use eyre::{eyre, Context, Result};
use lofty::{file::TaggedFileExt as _, picture::Picture, read_from_path, tag::Tag};
use sanitize_filename::sanitize;
use tracing::warn;

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
        track: Some(this_track.position),
        total_tracks: Some(this_media.track_count),
        disc: Some(this_media.position),
        total_discs: Some(release.media.len() as u32),
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
        media: this_media.format.clone(),
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
    let mut tag = tagged_file
        .primary_tag()
        .cloned()
        .unwrap_or_else(|| Tag::new(tagged_file.primary_tag_type()));
    tag.retain(|item| {
        if let lofty::tag::ItemKey::Unknown(key) = item.key() {
            // lofty doesn't support ASIN tag and if tag has it, lofty throws error.
            if key == "ASIN" {
                warn!("ASIN tag found. Removing...");
                return false;
            }
        }
        true
    });
    Ok(tag)
}

/// Determine the save path from metadata and source path (for detecting extension).
pub(super) fn get_save_path_from_metadata(
    source_path: &Path,
    target_dir: &Path,
    metadata: &Metadata,
) -> eyre::Result<PathBuf> {
    let Some(Some(ext)) = source_path.extension().map(|ext| ext.to_str()) else {
        return Err(eyre!("No extension found!"));
    };

    let mut new_path = PathBuf::new();

    let artist = metadata
        .artist
        .as_ref()
        .ok_or_else(|| eyre!("artist not found"))?;
    let album = metadata
        .album
        .as_ref()
        .ok_or_else(|| eyre!("album not found"))?;
    let title = metadata
        .title
        .as_ref()
        .ok_or_else(|| eyre!("title not found"))?;
    let album_artist = &metadata.album_artist;
    let track = &metadata.track;

    new_path.push(target_dir);
    new_path.push(sanitize(album_artist.clone().unwrap_or(artist.clone())));
    new_path.push(sanitize(album.clone()));
    if let (Some(total_discs), Some(disc)) = (metadata.total_discs, metadata.disc) {
        if total_discs > 1 {
            let width = total_discs.to_string().len();
            let disc = format!("{:0width$}", disc, width = width);
            new_path.push(sanitize(format!("Disc {}", disc)));
        }
    }
    new_path.push(sanitize({
        let mut file_name = String::new();
        if let Some(track) = track {
            let width = metadata.total_tracks.unwrap_or(0).to_string().len();
            let track = format!("{:0width$}", track, width = width);
            file_name.push_str(&track);
            file_name.push_str(" - ");
        }
        file_name.push_str(title);
        file_name.push('.');
        file_name.push_str(ext);
        file_name
    }));
    Ok(new_path)
}

#[cfg(test)]
mod test {
    use std::path::PathBuf;

    #[tokio::test]
    async fn cover_art() {
        let cover_art = super::fetch_cover_art("db85c244-53e7-441c-bab0-52c9c0d27450")
            .await
            .unwrap();
        assert_eq!(
            cover_art.mime_type().map(|s| s.to_string()),
            Some("image/jpeg".to_string())
        );
    }

    #[test]
    fn save_path_1() {
        let get_path = |metadata: &super::Metadata| {
            super::get_save_path_from_metadata(
                &std::path::PathBuf::from("source_path.mp3"),
                &PathBuf::from("/target_dir"),
                metadata,
            )
            .unwrap()
        };
        let mut metadata = super::Metadata {
            title: Some("title".to_string()),
            artist: Some("artist".to_string()),
            artist_sort: Some("artist_sort".to_string()),
            album: Some("album".to_string()),
            album_artist: Some("album_artist".to_string()),
            album_artist_sort: Some("album_artist_sort".to_string()),
            track: Some(1),
            total_tracks: Some(2),
            disc: Some(1),
            total_discs: Some(2),
            original_date: Some("original_date".to_string()),
            date: Some("date".to_string()),
            year: Some("year".to_string()),
            label: Some("label".to_string()),
            media: Some("media".to_string()),
            script: Some("script".to_string()),
            musicbrainz_artist_id: Some("musicbrainz_artist_id".to_string()),
            musicbrainz_track_id: Some("musicbrainz_track_id".to_string()),
            musicbrainz_release_id: Some("musicbrainz_release_id".to_string()),
            musicbrainz_release_artist_id: Some("musicbrainz_release_artist_id".to_string()),
            musicbrainz_release_group_id: Some("musicbrainz_release_group_id".to_string()),
            musicbrainz_recording_id: Some("musicbrainz_recording_id".to_string()),
        };
        assert_eq!(
            get_path(&metadata),
            std::path::PathBuf::from("/target_dir/album_artist/album/Disc 1/1 - title.mp3")
        );

        metadata.total_tracks = Some(10);
        assert_eq!(
            get_path(&metadata),
            std::path::PathBuf::from("/target_dir/album_artist/album/Disc 1/01 - title.mp3")
        );

        metadata.total_discs = None;
        assert_eq!(
            get_path(&metadata),
            std::path::PathBuf::from("/target_dir/album_artist/album/01 - title.mp3")
        );

        metadata.total_discs = Some(10);
        assert_eq!(
            get_path(&metadata),
            std::path::PathBuf::from("/target_dir/album_artist/album/Disc 01/01 - title.mp3")
        );
    }
}
