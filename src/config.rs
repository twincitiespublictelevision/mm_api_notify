extern crate log;
extern crate toml;

use std::collections::BTreeMap;
use std::fs::File;
use std::io::Read;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Config {
    pub db: DBConfig,
    pub mm: APIConfig,
    pub thread_pool_size: usize,
    pub min_runtime_delta: i64,
    pub lookback_timeframe: i64,
    pub ignore_skip: bool,
    pub log: LogConfig,
    pub enable_hooks: bool,
    pub hooks: Option<HookConfig>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct LogConfig {
    pub location: Option<String>,
    pub level: Option<String>,
}

// Database configuration/
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DBConfig {
    pub host: String,
    pub port: u16,
    pub name: String,
    pub username: String,
    pub password: String,
}

// MediaManagerAPI configuration
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct APIConfig {
    pub key: String,
    pub secret: String,
    pub env: Option<String>,
    pub changelog_max_timespan: i64,
}

// API Webhook configuration
pub type HookConfig = BTreeMap<String, Vec<BTreeMap<String, String>>>;

pub fn parse_config(path: &str) -> Option<Config> {
    let mut config_toml = String::new();

    let mut file = match File::open(path) {
        Ok(file) => file,
        Err(_) => {
            error!(
                "Could not find config file at {}. See the included README and \
                 config.toml.example for configuration instructions.",
                path
            );
            return None;
        }
    };

    file.read_to_string(&mut config_toml)
        .or_else(|err| {
            error!("Failure while reading config: [{}]", err);
            Err(err)
        })
        .ok()
        .and_then(|_| toml::from_str(&config_toml).ok())
}
