use crate::{
    config::{Config, CONFIG},
    router::Error,
};

use super::AppState;

pub async fn config_read(_: AppState, _: ()) -> Result<String, Error> {
    let config = toml::to_string_pretty(&*CONFIG.read())
        .map_err(|e| Error::Internal(format!("Failed to serialize config: {}", e,)))?;
    Ok(config)
}

pub async fn config_write(_: AppState, new_config: String) -> Result<(), Error> {
    let new_config: Config = toml::from_str(&new_config)
        .map_err(|e| Error::BadRequest(format!("Failed to deserialize config: {}", e,)))?;
    *CONFIG.write() = new_config;
    Ok(())
}
