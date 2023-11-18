use crate::config::{Config, CONFIG};

use super::AppState;

pub async fn config_read(_: AppState, _: ()) -> Result<String, rspc::Error> {
    let config = toml::to_string_pretty(&*CONFIG.read()).map_err(|e| {
        rspc::Error::new(
            rspc::ErrorCode::InternalServerError,
            format!("Failed to serialize config: {}", e,),
        )
    })?;
    Ok(config)
}

pub async fn config_write(_: AppState, new_config: String) -> Result<(), rspc::Error> {
    let new_config: Config = toml::from_str(&new_config).map_err(|e| {
        rspc::Error::new(
            rspc::ErrorCode::BadRequest,
            format!("Failed to deserialize config: {}", e,),
        )
    })?;
    *CONFIG.write() = new_config;
    Ok(())
}
