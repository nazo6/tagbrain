use lofty::{ItemKey, ItemValue, Tag, TagItem};
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Serialize, Deserialize, rspc::Type)]
pub struct Metadata {
    pub title: Option<String>,
    pub artist: Option<String>,
    pub artist_sort: Option<String>,
    pub album: Option<String>,
    pub album_artist: Option<String>,
    pub album_artist_sort: Option<String>,
    pub track: Option<String>,
    pub total_tracks: Option<String>,
    pub disk: Option<String>,
    pub total_disks: Option<String>,
    pub original_date: Option<String>,
    pub date: Option<String>,
    pub year: Option<String>,
    pub label: Option<String>,
    pub media: Option<String>,
    pub script: Option<String>,
    pub musicbrainz_track_id: Option<String>,
    pub musicbrainz_recording_id: Option<String>,
    pub musicbrainz_artist_id: Option<String>,
    pub musicbrainz_release_id: Option<String>,
    pub musicbrainz_release_artist_id: Option<String>,
    pub musicbrainz_release_group_id: Option<String>,
}

macro_rules! get {
    ($tag:expr, $key:ident) => {
        $tag.get_string(&ItemKey::$key).map(|s| s.to_owned())
    };
}

impl Metadata {
    pub fn from_tag(tag: &Tag) -> Self {
        Self {
            title: get!(tag, TrackTitle),
            artist: get!(tag, TrackArtist),
            artist_sort: get!(tag, TrackArtistSortOrder),
            album: get!(tag, AlbumTitle),
            album_artist: get!(tag, AlbumArtist),
            album_artist_sort: get!(tag, AlbumArtistSortOrder),
            track: get!(tag, TrackNumber),
            total_tracks: get!(tag, TrackTotal),
            disk: get!(tag, DiscNumber),
            total_disks: get!(tag, DiscTotal),
            original_date: get!(tag, OriginalReleaseDate),
            date: get!(tag, RecordingDate),
            year: get!(tag, Year),
            label: get!(tag, Label),
            media: get!(tag, OriginalMediaType),
            script: get!(tag, Script),
            musicbrainz_artist_id: get!(tag, MusicBrainzArtistId),
            musicbrainz_track_id: get!(tag, MusicBrainzTrackId),
            musicbrainz_release_id: get!(tag, MusicBrainzReleaseId),
            musicbrainz_release_artist_id: get!(tag, MusicBrainzReleaseArtistId),
            musicbrainz_release_group_id: get!(tag, MusicBrainzReleaseGroupId),
            musicbrainz_recording_id: get!(tag, MusicBrainzRecordingId),
        }
    }
}

macro_rules! insert {
    ($tag:expr, $key:ident, $value:expr) => {
        if let Some(value) = $value {
            $tag.insert(TagItem::new(ItemKey::$key, ItemValue::Text(value)));
        }
    };
}

#[rustfmt::skip]
pub fn write_metadata(tag: &mut Tag, metadata: Metadata) {
    insert!(tag, TrackTitle,                 metadata.title);
    insert!(tag, TrackArtist,                metadata.artist);
    insert!(tag, TrackArtistSortOrder,       metadata.artist_sort);
    insert!(tag, AlbumTitle,                 metadata.album);
    insert!(tag, AlbumArtist,                metadata.album_artist);
    insert!(tag, AlbumArtistSortOrder,       metadata.album_artist_sort);
    insert!(tag, TrackNumber,                metadata.track);
    insert!(tag, TrackTotal,                 metadata.total_tracks);
    insert!(tag, DiscNumber,                 metadata.disk);
    insert!(tag, DiscTotal,                  metadata.total_disks);
    insert!(tag, OriginalReleaseDate,        metadata.original_date);
    insert!(tag, RecordingDate,              metadata.date);
    insert!(tag, Year,                       metadata.year);
    insert!(tag, Label,                      metadata.label);
    insert!(tag, OriginalMediaType,          metadata.media);
    insert!(tag, Script,                     metadata.script);
    insert!(tag, MusicBrainzArtistId,        metadata.musicbrainz_artist_id);
    insert!(tag, MusicBrainzTrackId,         metadata.musicbrainz_track_id);
    insert!(tag, MusicBrainzReleaseId,       metadata.musicbrainz_release_id);
    insert!(tag, MusicBrainzReleaseArtistId, metadata.musicbrainz_release_artist_id);
    insert!(tag, MusicBrainzReleaseGroupId,  metadata.musicbrainz_release_group_id);
    insert!(tag, MusicBrainzRecordingId,     metadata.musicbrainz_recording_id);
}
