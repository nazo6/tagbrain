use crate::api::deserialize;

use super::{ArtistCredit, MusicbrainzClient};

#[derive(serde::Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct ReleaseRes {
    pub id: String,
    pub label_info: Option<Vec<ReleaseResLabelInfo>>,
    pub artist_credit: Option<Vec<ArtistCredit>>,
    pub media: Vec<ReleaseResMedia>,
    pub title: String,
    pub text_representation: Option<ReleaseResTextRepresentation>,
    pub date: Option<String>,
    pub disambiguation: Option<String>,
    pub release_group: ReleaseResReleaseGroup,
}

#[derive(serde::Deserialize, Debug, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct ReleaseResReleaseGroup {
    pub id: String,
    pub title: String,
    pub primary_type: Option<String>,
    pub first_release_date: Option<String>,
}

#[derive(serde::Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct ReleaseResTextRepresentation {
    pub script: Option<String>,
    pub language: Option<String>,
}

#[derive(serde::Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct ReleaseResLabelInfo {
    // pub catalog_number: String,
    pub label: Option<ReleaseResLabelInfoLabel>,
}
#[derive(serde::Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct ReleaseResLabelInfoLabel {
    pub name: String,
    pub id: String,
}

/// Represent of media like CD
#[derive(serde::Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct ReleaseResMedia {
    pub position: u32,
    pub format: Option<String>,
    pub tracks: Vec<ReleaseResMediaTrack>,
    pub track_count: u32,
}
#[derive(serde::Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct ReleaseResMediaTrack {
    pub number: String,
    pub title: String,
    pub id: String,
    pub position: u32,
    pub recording: ReleaseResMediaTrackRecording,
}
#[derive(serde::Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct ReleaseResMediaTrackRecording {
    pub id: String,
}

impl MusicbrainzClient {
    #[tracing::instrument(skip(self))]
    pub async fn release(&self, id: &str) -> Result<ReleaseRes, eyre::Error> {
        let url = format!("https://musicbrainz.org/ws/2/release/{}", id);
        let url = url::Url::parse_with_params(
            &url,
            &[
                ("fmt", "json"),
                ("inc", "artists+recordings+labels+release-groups"),
            ],
        )?;
        let text = self.get(url).await?.text().await?;
        let res: ReleaseRes = deserialize(&text)?;
        Ok(res)
    }
}
