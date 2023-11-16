use crate::config::CONFIG;
pub mod recording;
pub mod release;

pub struct MusicbrainzClient {
    client: reqwest::Client,
    semaphore: tokio::sync::Semaphore,
}

impl MusicbrainzClient {
    pub fn new() -> Self {
        let client = reqwest::ClientBuilder::new()
            .user_agent(&CONFIG.read().app_ua)
            .build()
            .unwrap();
        Self {
            client,
            semaphore: tokio::sync::Semaphore::new(1),
        }
    }
    async fn get(&self, url: url::Url) -> Result<reqwest::Response, reqwest::Error> {
        let res = self.client.get(url).send().await;

        // musicbrainz api rate limit is 1 request per second...
        // so we need to wait 1 second before next request and we use semaphore to disable parallel request
        let _permit = self.semaphore.acquire().await;
        tokio::time::sleep(std::time::Duration::from_millis(1000)).await;
        res
    }
}

#[derive(serde::Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct ArtistCredit {
    pub artist: ArtistCreditArtist,
    pub joinphrase: String,
}
#[derive(serde::Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct ArtistCreditArtist {
    pub id: String,
    pub sort_name: String,
    pub name: String,
}

pub trait ArtistCreditVecToString {
    fn to_string(&self) -> String;
    fn to_sort_string(&self) -> String;
}
impl ArtistCreditVecToString for Vec<ArtistCredit> {
    fn to_string(&self) -> String {
        self.iter().fold(String::new(), |mut acc, ac| {
            acc.push_str(&ac.artist.name);
            acc.push_str(&ac.joinphrase);
            acc
        })
    }
    fn to_sort_string(&self) -> String {
        self.iter().fold(String::new(), |mut acc, ac| {
            acc.push_str(&ac.artist.sort_name);
            acc.push_str(&ac.joinphrase);
            acc
        })
    }
}
