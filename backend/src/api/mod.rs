use serde::de::DeserializeOwned;

#[allow(dead_code)]
pub mod acoustid;
#[allow(dead_code)]
pub mod musicbrainz;

fn deserialize<T: DeserializeOwned>(
    str: &str,
) -> Result<T, serde_path_to_error::Error<serde_json::Error>> {
    let jd = &mut serde_json::Deserializer::from_str(str);

    let result: Result<T, _> = serde_path_to_error::deserialize(jd);

    result
}
