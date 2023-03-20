use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

use crate::error::Error;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Config {
    pub path: PathBuf,
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
        let path = find_config_file(config_path.clone()).unwrap();

        let config = config::Config::builder()
            .add_source(config::Environment::with_prefix("PUP"))
            .add_source(config::File::from(find_config_file(config_path).unwrap()))
            .build()
            .unwrap();

        let mut modules = config
            .try_deserialize::<HashMap<String, ConfigModule>>()
            .unwrap();

        // Expand paths.
        for (_, module) in modules.iter_mut() {
            module.install_dir = shellexpand::full(&module.install_dir.to_str().unwrap())
                .unwrap()
                .to_string()
                .into();
            module.cache_dir = shellexpand::full(&module.cache_dir.to_str().unwrap())
                .unwrap()
                .to_string()
                .into();
        }

        Self { path, modules }
    }
}

fn find_config_file(config_path: Option<String>) -> Result<PathBuf, Error> {
    let config_path = match config_path {
        Some(path) => PathBuf::from(path),
        None => {
            let path = dirs::config_dir()
                .ok_or(Error::NotFound("config dir".to_string()))?
                .join("pup")
                .join("config.toml");
            path
        }
    };

    if !config_path.exists() {
        info!(
            "Config file not found, creating default config at {:?}",
            config_path
        );

        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        create_default_config(config_path.clone())?;
    }

    Ok(config_path)
}

fn create_default_config(path: PathBuf) -> Result<(), Error> {
    let default_config = include_str!("../config.default.toml");
    let mut file = File::create(path)?;
    file.write_all(default_config.as_bytes())?;
    Ok(())
}
