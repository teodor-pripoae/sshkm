pub mod app;
pub mod config;

use std::{env, str::Utf8Error};

use log::error;
use seahorse::{App, Command, Context, Flag, FlagType};
use thiserror::Error;

use crate::config::Config;

#[derive(Error, Debug)]
pub(crate) enum Error {
    #[error("Config error: {0}")]
    ConfigError(#[from] crate::config::ConfigError),
    #[error("HTTP error: {0}")]
    HTTPError(#[from] reqwest::Error),
    #[error("IO error: {0}")]
    IOError(#[from] std::io::Error),
    #[error("Failed to set owner: {0}")]
    SetOwnerError(#[from] file_owner::FileOwnerError),
    #[error("Path error")]
    PathError,
    #[error("User not found: {0}")]
    UserNotFound(String),
    #[error("utf8 error: {0}")]
    Utf8Error(#[from] Utf8Error),
    #[error("null error: {0}")]
    NullErrro(#[from] std::ffi::NulError),
}

fn default_action(c: &Context) {
    c.help();
}

fn daemon_cmd() -> Command {
    Command::new("daemon")
        .description("Run as a daemon")
        .usage("sshkm daemon --interval 60")
        .flag(
            Flag::new("config", FlagType::String)
                .description("Path to the config file")
                .alias("c"),
        )
        .flag(
            Flag::new("interval", FlagType::Uint)
                .description("Interval in seconds")
                .alias("i"),
        )
        .action(daemon)
}

#[tokio::main]
async fn daemon(c: &Context) {
    let config_path = c
        .string_flag("config")
        .unwrap_or_else(|_| String::from("config.yaml"));

    let config = Config::from_file(&config_path).expect("failed to parse config file");

    let interval = c
        .uint_flag("interval")
        .unwrap_or(config.interval() as usize) as u64;
    let interval_msecs: u64 = interval * 1000;

    let app = crate::app::App::new(config);

    loop {
        let start_time = std::time::Instant::now();
        if let Err(e) = app.run_once().await {
            error!("Error running: {}", e);
        }

        let elapsed = start_time.elapsed().as_millis() as u64;

        if elapsed < interval_msecs {
            tokio::time::sleep(std::time::Duration::from_millis(interval_msecs - elapsed)).await;
        }
    }
}

fn sync_cmd() -> Command {
    Command::new("sync")
        .description("Sync the ssh keys once")
        .usage("sshkm sync")
        .flag(
            Flag::new("config", FlagType::String)
                .description("Path to the config file")
                .alias("c"),
        )
        .action(sync)
}

#[tokio::main]
async fn sync(c: &Context) {
    let config_path = c
        .string_flag("config")
        .unwrap_or_else(|_| String::from("config.yaml"));
    let config = Config::from_file(&config_path).expect("Failed to load config");

    let app = app::App::new(config);

    app.run_once().await.expect("Failed to run");
}

fn main() {
    env_logger::init_from_env(
        env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "info"),
    );

    let args: Vec<String> = env::args().collect();
    let app = App::new(env!("CARGO_PKG_NAME"))
        .description(env!("CARGO_PKG_DESCRIPTION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .version(env!("CARGO_PKG_VERSION"))
        .usage("sshkm [name]")
        .action(default_action)
        .command(daemon_cmd())
        .command(sync_cmd());

    app.run(args);
}
