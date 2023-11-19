use serde::de::DeserializeOwned;

pub mod acoustid;
pub mod musicbrainz;

fn deserialize<T: DeserializeOwned>(
    str: &str,
) -> Result<T, serde_path_to_error::Error<serde_json::Error>> {
    let jd = &mut serde_json::Deserializer::from_str(str);

    let result: Result<T, _> = serde_path_to_error::deserialize(jd);

    result
}
