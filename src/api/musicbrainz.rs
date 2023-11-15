use crate::config::CONFIG;
pub mod recording;
pub mod release;

pub struct MusicbrainzClient {
    client: reqwest::Client,
}

impl MusicbrainzClient {
    pub fn new() -> Self {
        let client = reqwest::ClientBuilder::new()
            .user_agent(&CONFIG.read().app_ua)
            .build()
            .unwrap();
        Self { client }
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
