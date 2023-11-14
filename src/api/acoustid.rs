use crate::config::CONFIG;

pub struct AcoustidClient {
    client: reqwest::Client,
}

#[derive(serde::Deserialize, Debug)]
pub struct RookupRes {
    pub results: Vec<RookupResEntry>,
}
#[derive(serde::Deserialize, Debug)]
pub struct RookupResEntry {
    pub id: String,
    pub recordings: Vec<ReookupResEntryRecording>,
    pub score: f64,
}
#[derive(serde::Deserialize, Debug)]
pub struct ReookupResEntryRecording {
    pub id: String,
}

impl AcoustidClient {
    pub fn new() -> Self {
        let client = reqwest::ClientBuilder::new()
            .user_agent(&CONFIG.read().app_ua)
            .build()
            .unwrap();
        Self { client }
    }

    pub async fn lookup(
        &self,
        fingerprint: &str,
        duration: i64,
    ) -> Result<RookupRes, anyhow::Error> {
        let url = "https://api.acoustid.org/v2/lookup";
        let url = url::Url::parse_with_params(
            url,
            &[
                ("client", &CONFIG.read().acoust_id_api_key),
                ("meta", &"recordingids".to_string()),
                ("duration", &duration.to_string()),
                ("fingerprint", &fingerprint.to_string()),
            ],
        )?;
        let res: RookupRes = self.client.get(url).send().await?.json().await?;
        Ok(res)
    }
}
