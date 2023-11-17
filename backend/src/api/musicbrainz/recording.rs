use super::{ArtistCredit, MusicbrainzClient};

#[derive(serde::Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct RecordingRes {
    pub title: String,
    pub id: String,
    pub releases: Vec<RecordingResRelease>,
    pub artist_credit: Option<Vec<ArtistCredit>>,
    pub first_release_date: Option<String>,
}
#[derive(serde::Deserialize, Debug, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct RecordingResRelease {
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
    pub first_release_date: Option<String>,
}

#[derive(serde::Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct RecordingSearchRes {
    pub recordings: Vec<RecordingRes>,
}

impl MusicbrainzClient {
    #[tracing::instrument(skip(self))]
    pub async fn recording(&self, id: &str) -> Result<RecordingRes, eyre::Report> {
        let url = format!("https://musicbrainz.org/ws/2/recording/{}", id);
        let url = url::Url::parse_with_params(
            &url,
            &[("fmt", "json"), ("inc", "releases+release-groups+artists")],
        )?;
        let res: RecordingRes = self.client.get(url).send().await?.json().await?;
        // let text = self.get(url).await?.text().await?;
        // let debug_res: serde_json::Value = serde_json::from_str(&text)?;
        // let res = serde_json::from_str::<RecordingRes>(&text);
        // if let Err(e) = &res {
        //     warn!("recording dbg {}: {:?}", id, debug_res);
        // }
        Ok(res)
    }
    pub async fn recording_search(&self, query: &str) -> Result<RecordingSearchRes, eyre::Report> {
        let url = "https://musicbrainz.org/ws/2/recording";
        let url = url::Url::parse_with_params(
            url,
            &[("fmt", "json"), ("query", query), ("limit", "15")],
        )?;
        let res: RecordingSearchRes = self.client.get(url).send().await?.json().await?;
        Ok(res)
    }
}
