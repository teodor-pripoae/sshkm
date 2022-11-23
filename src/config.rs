use serde::{Deserialize, Serialize};
use thiserror::Error;

const DEFAULT_GITHUB_URL: &str = "https://github.com";

#[derive(Error, Debug)]
pub(crate) enum ConfigError {
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("YAML error: {0}")]
    YAMLError(#[from] serde_yaml::Error),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct Config {
    pub github_url: Option<String>,
    pub users: Vec<User>,

    pub timeout: Option<u64>,
    pub interval: Option<u64>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct User {
    pub username: String,
    pub github_username: Option<String>,
}

impl Config {
    pub fn from_file(path: &str) -> Result<Self, ConfigError> {
        let contents = std::fs::read_to_string(path)?;
        let config = serde_yaml::from_str(&contents)?;
        Ok(config)
    }

    pub fn github_url(&self) -> &str {
        self.github_url.as_deref().unwrap_or(DEFAULT_GITHUB_URL)
    }

    pub fn timeout(&self) -> u64 {
        self.timeout.unwrap_or(10)
    }

    pub fn interval(&self) -> u64 {
        self.interval.unwrap_or(60)
    }
}
