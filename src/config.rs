use std::sync::RwLock;

use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use tracing::{error, info};

/// App configuration
#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub source_dir: String,
    pub target_dir: String,
    pub acoust_id_api_key: String,
    pub app_ua: String,
    pub acoustid_match_threshold: f64,
    pub release_selector: ReleaseSelector,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            source_dir: "./data/source".to_string(),
            target_dir: "./data/target".to_string(),
            acoust_id_api_key: std::env::var("ACOUST_ID_API_KEY")
                .expect("ACOUST_ID_API_KEY not set"),
            app_ua: concat!(
                "tagbrain",
                "/",
                env!("CARGO_PKG_VERSION"),
                " (",
                "https://github.com/nazo6",
                ")"
            )
            .to_string(),
            acoustid_match_threshold: 0.8,
            release_selector: ReleaseSelector::default(),
        }
    }
}

/// When we grab data from musicbrainz, we need to select the best match.
/// This struct defines the rules for that.
///
/// According to this setting, a score is calculated for each field for each release, and the release with the highest total score is selected.
///
/// The `preferred` can be an array. If the field matches any of the elements in the array, the score is added.
/// The `weight` specifies the weight of the field.
/// The score of `preferred` multiplied by the `weight` is the score of the field.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ReleaseSelector {
    /// ref: https://musicbrainz.org/doc/Release_Group/Type
    pub release_group_type: MatchReleaseSelector,
    pub country: MatchReleaseSelector,
    /// Read metadata from current file and calculate levenshtein distance.
    pub release_title_distance: DistanceReleaseSelector,
}

impl Default for ReleaseSelector {
    fn default() -> Self {
        Self {
            release_group_type: MatchReleaseSelector {
                /// ex: ["Album", "EP", "Single"]
                preferred: vec![],
                weight: 1.0,
            },
            country: MatchReleaseSelector {
                /// ex: ["US", "JP", "XW"]
                preferred: vec![],
                weight: 1.0,
            },
            release_title_distance: DistanceReleaseSelector { weight: 1.0 },
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct MatchReleaseSelector {
    pub preferred: Vec<String>,
    pub weight: f64,
}
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct DistanceReleaseSelector {
    pub weight: f64,
}

static CONFIG_PATH: Lazy<String> = Lazy::new(|| {
    std::env::var("CONFIG_PATH").unwrap_or_else(|_| "./config/config.toml".to_string())
});

pub struct ConfigWrapper {
    pub config: RwLock<Config>,
}

impl ConfigWrapper {
    pub fn read(&self) -> std::sync::RwLockReadGuard<Config> {
        self.config.read().unwrap()
    }
    pub fn write(&self) -> ConfigRwLockWriteGuardWrapper<'_> {
        ConfigRwLockWriteGuardWrapper {
            config: self.config.write().unwrap(),
        }
    }
}

pub struct ConfigRwLockWriteGuardWrapper<'a> {
    pub config: std::sync::RwLockWriteGuard<'a, Config>,
}
impl Drop for ConfigRwLockWriteGuardWrapper<'_> {
    fn drop(&mut self) {
        let config = toml::to_string_pretty(&*self.config).unwrap();
        tokio::spawn({
            async move {
                let res = tokio::fs::write(&*CONFIG_PATH, config).await;
                if let Err(e) = res {
                    error!("Failed to write config file: {}", e);
                }
                info!("Config file updated.");
            }
        });
    }
}

pub static CONFIG: Lazy<ConfigWrapper> = Lazy::new(|| {
    let config = if let Ok(config) = std::fs::read_to_string(&*CONFIG_PATH) {
        if let Ok(config) = toml::from_str::<Config>(&config) {
            info!("Config file read successfully.");
            config
        } else {
            error!("Failed to parse config file");
            panic!("Failed to parse config file");
        }
    } else {
        info!("Failed to read config file, using defaults.");
        let config = Config::default();
        let config_str = toml::to_string_pretty(&config).unwrap();
        std::fs::write(&*CONFIG_PATH, config_str).unwrap();
        config
    };
    ConfigWrapper {
        config: RwLock::new(config),
    }
});
