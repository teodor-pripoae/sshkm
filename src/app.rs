use std::{
    ffi::CString, fs::Permissions, os::unix::prelude::PermissionsExt, path::PathBuf, time::Duration,
};

use etc_passwd::Passwd;
use file_owner::PathExt;
use log::info;
use tokio::io::AsyncWriteExt;

use crate::config::Config;

#[derive(Debug, Clone)]
pub(crate) struct App {
    pub config: Config,
}

impl App {
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    pub async fn run_once(&self) -> Result<(), crate::Error> {
        for user in &self.config.users {
            let github_username = match &user.github_username {
                Some(username) => username,
                None => {
                    log::warn!(
                        "No github username for user {}, assuming same as the username",
                        user.username
                    );
                    &user.username
                }
            };

            let keys = self.get_ssh_keys(github_username).await?;
            self.write_keys(&user.username, keys).await?;
        }

        Ok(())
    }

    pub async fn get_ssh_keys(&self, github_username: &str) -> Result<Vec<String>, crate::Error> {
        let url = format!("{}/{}.keys", self.config.github_url(), github_username);

        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(self.config.timeout()))
            .build()?;

        let response = client.get(&url).send().await?;

        let keys = response.text().await?;

        Ok(keys.lines().map(|s| s.to_string()).collect())
    }

    pub async fn write_keys(&self, username: &str, keys: Vec<String>) -> Result<(), crate::Error> {
        let home = self.home_directory(username)?;
        let authorized_keys = home.join(".ssh").join("authorized_keys");
        let authorized_keys_file = authorized_keys.to_str().ok_or(crate::Error::PathError)?;

        info!("Writing keys for {} to {}", username, authorized_keys_file);

        let mut file = tokio::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .mode(0o600)
            .truncate(true)
            .open(authorized_keys_file)
            .await?;

        for key in keys {
            file.write_all(key.as_bytes()).await?;
            file.write_all(b"\n").await?;
        }

        file.sync_all().await?;

        file.set_permissions(Permissions::from_mode(0o600)).await?;

        authorized_keys_file.set_owner(username)?;
        authorized_keys_file.set_group(username)?;

        info!("Wrote keys for {} to {}", username, authorized_keys_file);

        Ok(())
    }

    fn home_directory(&self, username: &str) -> Result<std::path::PathBuf, crate::Error> {
        let passwd = Passwd::from_name(CString::new(username)?)?;

        match passwd {
            None => Err(crate::Error::UserNotFound(username.to_string())),
            Some(passwd) => {
                let dir = PathBuf::from(passwd.dir.to_str()?).canonicalize()?;
                Ok(dir)
            }
        }
    }
}
