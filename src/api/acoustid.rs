use crate::config::CONFIG;

pub struct AcoustidClient {
    client: reqwest::Client,
}

#[derive(serde::Deserialize, Debug)]
pub struct AcoustidLookupResult {
    pub results: Vec<AcoustidLookupResultEntry>,
}
#[derive(serde::Deserialize, Debug)]
pub struct AcoustidLookupResultEntry {
    pub id: String,
    pub recordings: Vec<AcoustidLookupResultRecordingid>,
    pub score: f64,
}
#[derive(serde::Deserialize, Debug)]
pub struct AcoustidLookupResultRecordingid {
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
    ) -> Result<AcoustidLookupResult, anyhow::Error> {
        let url = "https://api.acoustid.org/v2/lookup";
        let url = url::Url::parse_with_params(
            url,
            &[
                ("client", &CONFIG.read().app_ua),
                ("meta", &"recordingids".to_string()),
                ("duration", &duration.to_string()),
                ("fingerprint", &fingerprint.to_string()),
            ],
        )?;
        let res: AcoustidLookupResult = self.client.get(url).send().await?.json().await?;
        Ok(res)
    }
}
