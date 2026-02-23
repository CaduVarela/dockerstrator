use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub max_depth: Option<usize>,
    pub excluded_dirs: Vec<String>,
    #[serde(default)]
    pub legacy_compose: bool,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            max_depth: Some(7),
            excluded_dirs: Vec::new(),
            legacy_compose: false,
        }
    }
}

pub fn get_config_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home)
        .join(".config")
        .join("dockerstrator")
        .join("config.toml")
}

pub fn load_config() -> Config {
    let config_path = get_config_path();

    if config_path.exists() {
        if let Ok(contents) = fs::read_to_string(&config_path) {
            if let Ok(config) = toml::from_str(&contents) {
                return config;
            }
        }
    }

    Config::default()
}

pub fn save_config(config: &Config) -> Result<(), String> {
    let config_path = get_config_path();

    if let Some(parent) = config_path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }

    let contents = toml::to_string_pretty(config).map_err(|e| e.to_string())?;
    fs::write(&config_path, contents).map_err(|e| e.to_string())?;

    Ok(())
}
