use crate::{
    api::musicbrainz::{
        recording::{RecordingRes, RecordingResRelease},
        release::ReleaseRes,
        ArtistCreditVecToString,
    },
    interface::metadata::Metadata,
};
use eyre::Result;

/// Collect data, and format it into a metadata struct.
pub(super) fn format_to_metadata(
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
            let mut album = release_additional.title.clone();
            if let Some(disambiguation) = release_additional.disambiguation {
                if !disambiguation.is_empty() {
                    album.push_str(" (");
                    album.push_str(&disambiguation);
                    album.push(')');
                }
            }
            album
        }),
        album_artist: release_additional
            .artist_credit
            .as_ref()
            .map(|a| a.to_string()),
        album_artist_sort: release_additional
            .artist_credit
            .as_ref()
            .map(|a| a.to_sort_string()),
        track: Some(this_track.position.to_string()),
        total_tracks: Some(this_media.track_count.to_string()),
        disk: Some(this_media.position.to_string()),
        total_disks: Some(release_additional.media.len().to_string()),
        original_date: release.release_group.first_release_date,
        date: release_additional.date.clone(),
        year: release_additional
            .date
            .and_then(|d| d.split('-').next().map(|s| s.to_owned())),
        label: release_additional.label_info.and_then(|label| {
            label
                .first()
                .and_then(|li| li.label.as_ref().map(|label| label.name.clone()))
        }),
        media: Some(this_media.format.clone()),
        script: release_additional
            .text_representation
            .and_then(|tr| tr.script),
        musicbrainz_artist_id: recording
            .artist_credit
            .and_then(|ac| ac.first().map(|ac| ac.artist.id.clone())),
        musicbrainz_track_id: Some(this_track.id.clone()),
        musicbrainz_release_id: Some(release.id),
        musicbrainz_release_artist_id: release_additional
            .artist_credit
            .and_then(|ac| ac.first().map(|ac| ac.artist.id.clone())),
        musicbrainz_release_group_id: Some(release.release_group.id),
        musicbrainz_recording_id: Some(recording.id),
    };

    Ok(metadata)
}
