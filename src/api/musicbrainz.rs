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
}
