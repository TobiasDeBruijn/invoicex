use std::default::default;
use std::path::{Path, PathBuf};
use std::process::exit;
use serde::{Deserialize, Serialize};
use tokio::fs;
use anyhow::Result;
use rand::Rng;
use serde::de::Unexpected::Str;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tracing::{info, trace, warn};

#[cfg(not(debug_assertions))]
const CFG_FOLDER: &str = "/etc/invoicex/";
#[cfg(debug_assertions)]
const CFG_FOLDER: &str = ".";

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct Config {
    pub mysql: MysqlConfig,
    pub http: HttpConfig,
    pub security: SecurityConfig,
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct MysqlConfig {
    pub host: String,
    pub database: String,
    pub username: String,
    pub password: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct HttpConfig {
    pub port: u16,
    pub frontend_host: String,
}

impl Default for HttpConfig {
    fn default() -> Self {
        Self {
            port: 8080,
            frontend_host: String::default()
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SecurityConfig {
    pub password_pepper: String,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            password_pepper: rand::thread_rng().sample_iter(rand::distributions::Alphanumeric).take(64).map(char::from).collect()
        }
    }
}

impl Config {
    pub async fn new() -> Result<Self> {
        let path = PathBuf::from(CFG_FOLDER).join("config.toml");

        trace!("Checking if configuration file at {path:?} exists");
        if !path.exists() {
            warn!("No configuration exists at {path:?}. Creating default'");
            Self::create_default_config(&path).await?;
            info!("Default configuration created. Exiting");
            exit(0);
        }
        trace!("Configuration exists");

        trace!("Opening configuration file at {path:?}");
        let mut file = fs::File::open(&file_path).await?;

        trace!("Reading configuration file");
        let mut buf = Vec::default();
        file.read_to_end(&mut buf).await?;

        trace!("Deserializing configuration");
        let this: Self = toml::from_slice(&buf)?;

        Ok(this)
    }

    async fn create_default_config(path: &Path) -> Result<Self> {
        let this = Self::default();

        trace!("Serializing default config");
        let serialized = toml::to_string_pretty(&this)?;

        trace!("Creating config file at {path:?}");
        let mut file = fs::File::create(path).await?;
        trace!("Writing default config to file");
        file.write_all(&serialized.as_bytes()).await?;

        Ok(this)
    }
}