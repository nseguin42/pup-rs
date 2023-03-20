
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

use crate::error::Error;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Config {
    pub modules: HashMap<String, ConfigModule>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ConfigModule {
    pub install_dir: PathBuf,
    pub cache_dir: PathBuf,
    pub repo: String,
    pub owner: String,
}

impl Config {
    pub fn new(config_path: Option<String>) -> Self {
        let config = config::Config::builder()
            .add_source(config::Environment::with_prefix("PUP"))
            .add_source(config::File::from(find_config_file(config_path).unwrap()))
            .build()
            .unwrap();

        let modules = config
            .try_deserialize::<HashMap<String, ConfigModule>>()
            .unwrap();

        Self { modules }
    }
}

fn find_config_file(config_path: Option<String>) -> Result<PathBuf, Error> {
    let config_path = match config_path {
        Some(path) => PathBuf::from(path),
        None => {
            let mut path = dirs::config_dir().ok_or(Error::NotFound("config dir".to_string()))?;
            path.push("pup");
            path.push("config.toml");
            path
        }
    };

    if !config_path.exists() {
        return Err(Error::NotFound(format!(
            "Config file not found at {}",
            config_path.to_str().unwrap()
        )));
    }

    Ok(config_path)
}
