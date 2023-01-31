use std::{fs, path::PathBuf};

use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub port: u16,
    pub index: Index,
    pub storage: Storage,
    #[serde(default)]
    pub tracing: Option<Tracing>,
}

#[derive(Debug, Deserialize)]
pub struct Index {
    pub location: PathBuf,
    pub config: IndexConfig,
}

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct IndexConfig {
    pub dl: String,
    pub api: String,
}

#[derive(Debug, Deserialize)]
pub struct Storage {
    pub location: PathBuf,
}

#[derive(Debug, Deserialize)]
pub struct Tracing {
    pub otlp: Otlp,
}

#[derive(Debug, Deserialize)]
pub struct Otlp {
    pub endpoint: String,
}

pub fn load() -> Result<Settings> {
    let locations = &[
        concat!("/etc/", env!("CARGO_PKG_NAME"), "/config.toml"),
        concat!("/app/", env!("CARGO_PKG_NAME"), ".toml"),
        concat!(env!("CARGO_PKG_NAME"), ".toml"),
    ];
    let buf = locations.iter().find_map(|loc| fs::read_to_string(loc).ok());

    match buf {
        Some(buf) => Ok(toml::from_str(&buf)?),
        None => bail!("failed finding settings"),
    }
}
