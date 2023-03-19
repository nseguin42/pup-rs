use crate::error::Error;
use std::path::PathBuf;

pub struct Config {
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

        let install_dir = config.get_string("install_dir").unwrap();
        let cache_dir = config
            .get::<Option<String>>("cache_dir")
            .unwrap()
            .unwrap_or(dirs::cache_dir().unwrap().to_str().unwrap().to_string());

        let repo = config.get_string("repo").unwrap();
        let owner = config.get_string("owner").unwrap();

        Self {
            install_dir: PathBuf::from(install_dir),
            cache_dir: PathBuf::from(cache_dir),
            repo,
            owner,
        }
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
