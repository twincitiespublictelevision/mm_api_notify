extern crate toml;

use std::collections::BTreeMap;
use std::fs::File;
use std::io::Read;

use self::toml::{Parser, Value};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Config {
    pub db: DBConfig,
    pub mm: MMConfig,
    pub thread_pool_size: usize,
    pub min_runtime_delta: i64,
    pub enable_hooks: bool,
    pub hooks: Option<APIConfig>,
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
pub struct MMConfig {
    pub key: String,
    pub secret: String,
    pub changelog_max_timespan: i64,
}

// API Webhook configuration
pub type APIConfig = BTreeMap<String, Vec<String>>;

pub fn parse_config(path: &str) -> Option<Config> {
    let mut config_toml = String::new();

    let mut file = match File::open(path) {
        Ok(file) => file,
        Err(_) => {
            println!("Error: Could not find config file (config.toml) at {}",
                     path);
            return None;
        }
    };

    file.read_to_string(&mut config_toml)
        .unwrap_or_else(|err| panic!("Error while reading config: [{}]", err));

    let mut parser = Parser::new(&config_toml);
    let toml = parser.parse();

    match toml {
        None => {
            for err in &parser.errors {
                let (loline, locol) = parser.to_linecol(err.lo);
                let (hiline, hicol) = parser.to_linecol(err.hi);
                println!("{}:{}:{}-{}:{} error: {}",
                         path,
                         loline,
                         locol,
                         hiline,
                         hicol,
                         err.desc);
            }
            panic!("Exiting server");
        }
        Some(value) => toml::decode(Value::Table(value)),
    }
}
