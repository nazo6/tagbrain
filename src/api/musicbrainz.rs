use crate::config::CONFIG;

pub struct MusicbrainzClient {
    client: reqwest::Client,
}

#[derive(serde::Deserialize, Debug)]
pub struct RecordingRes {
    pub title: String,
    pub id: String,
    pub releases: Vec<RecordingResRelease>,
}
#[derive(serde::Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct RecordingResRelease {
    pub date: Option<String>,
    pub id: String,
    pub country: Option<String>,
    pub release_group: RecordingResReleaseGroup,
}
#[derive(serde::Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct RecordingResReleaseGroup {
    pub id: String,
    pub title: String,
    pub primary_type: String,
}

#[derive(serde::Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct ReleaseRes {
    pub label_info: Vec<ReleaseResLabelInfo>,
    pub artist_credit: Vec<ReleaseResArtistCredit>,
    pub media: Vec<ReleaseResMedia>,
}
#[derive(serde::Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct ReleaseResLabelInfo {
    pub catalog_number: String,
    pub label: ReleaseResLabelInfoLabel,
}
#[derive(serde::Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct ReleaseResLabelInfoLabel {
    pub name: String,
    pub id: String,
}
#[derive(serde::Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct ReleaseResArtistCredit {
    pub name: String,
    pub sort_name: String,
}
#[derive(serde::Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct ReleaseResMedia {
    pub position: i64,
    pub format: String,
    pub tracks: Vec<ReleaseResMediaTrack>,
}
#[derive(serde::Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct ReleaseResMediaTrack {
    pub number: String,
    pub title: String,
    pub id: String,
    pub position: i64,
}

impl MusicbrainzClient {
    pub fn new() -> Self {
        let client = reqwest::ClientBuilder::new()
            .user_agent(&CONFIG.read().app_ua)
            .build()
            .unwrap();
        Self { client }
    }

    #[tracing::instrument(err, skip(self))]
    pub async fn recording(&self, id: &str) -> Result<RecordingRes, anyhow::Error> {
        let url = format!("https://musicbrainz.org/ws/2/recording/{}", id);
        let url = url::Url::parse_with_params(
            &url,
            &[("fmt", "json"), ("inc", "releases+release-groups")],
        )?;
        let res: RecordingRes = self.client.get(url).send().await?.json().await?;
        Ok(res)
    }
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
