use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    #[serde(default = "default_path")]
    pub path: String,
    #[serde(default = "default_date_format")]
    pub date_format: String,
    #[serde(default = "default_habits")]
    pub habits: Vec<String>,
}

fn default_path() -> String {
    home::home_dir()
        .expect("Could not find home directory")
        .join(".config")
        .join("todo")
        .to_str()
        .unwrap()
        .to_string()
}

fn default_date_format() -> String {
    "%Y-%m-%d".to_string()
}

fn default_habits() -> Vec<String> {
    vec![]
}

impl Default for Config {
    fn default() -> Self {
        Config {
            path: default_path(),
            date_format: default_date_format(),
            habits: default_habits(),
        }
    }
}

impl Config {
    pub fn parse() -> Result<Config> {
        let xdg_config_home = std::env::var("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .unwrap_or(
                std::env::var("HOME")
                    .map(|home| Path::new(&home).join(".config"))
                    .unwrap_or(PathBuf::from(".config")),
            );

        let config_path = xdg_config_home.join("todo").join("config.json");

        if !config_path.exists() {
            return Ok(Config::default());
        }

        return Config::from_file(&config_path);
    }

    pub fn from_file<P>(path: P) -> Result<Config>
    where
        P: AsRef<Path>,
    {
        let config_file = std::fs::File::open(path)?;

        return serde_json::from_reader(config_file).map_err(|e| e.into());
    }
}
