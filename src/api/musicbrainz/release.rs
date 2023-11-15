use super::{ArtistCredit, MusicbrainzClient};

#[derive(serde::Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct ReleaseRes {
    pub label_info: Option<Vec<ReleaseResLabelInfo>>,
    pub artist_credit: Vec<ArtistCredit>,
    pub media: Vec<ReleaseResMedia>,
    pub title: String,
}

#[derive(serde::Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct ReleaseResLabelInfo {
    pub catalog_number: String,
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
    pub format: String,
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
    #[tracing::instrument(skip(self), err)]
    pub async fn release(&self, id: &str) -> Result<ReleaseRes, anyhow::Error> {
        let url = format!("https://musicbrainz.org/ws/2/release/{}", id);
        let url = url::Url::parse_with_params(
            &url,
            &[("fmt", "json"), ("inc", "artists+recordings+labels")],
        )?;
        let res: ReleaseRes = self.client.get(url).send().await?.json().await?;
        Ok(res)
    }
}
