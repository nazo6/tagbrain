use super::{ArtistCredit, MusicbrainzClient};

#[derive(serde::Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct RecordingRes {
    pub title: String,
    pub id: String,
    pub releases: Vec<RecordingResRelease>,
    pub artist_credit: Vec<ArtistCredit>,
    pub first_release_date: String,
}
#[derive(serde::Deserialize, Debug, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct RecordingResRelease {
    pub date: Option<String>,
    pub id: String,
    pub country: Option<String>,
    pub release_group: RecordingResReleaseGroup,
}
#[derive(serde::Deserialize, Debug, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct RecordingResReleaseGroup {
    pub id: String,
    pub title: String,
    pub primary_type: Option<String>,
}

impl MusicbrainzClient {
    #[tracing::instrument(skip(self), err)]
    pub async fn recording(&self, id: &str) -> Result<RecordingRes, anyhow::Error> {
        let url = format!("https://musicbrainz.org/ws/2/recording/{}", id);
        let url = url::Url::parse_with_params(
            &url,
            &[("fmt", "json"), ("inc", "releases+release-groups+artists")],
        )?;
        // let res: RecordingRes = self.client.get(url).send().await?.json().await?;
        let text = self.client.get(url).send().await?.text().await?;
        let res: RecordingRes = serde_json::from_str(&text)?;
        Ok(res)
    }
}
