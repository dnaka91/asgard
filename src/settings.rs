use std::path::PathBuf;

use anyhow::Result;
use config::{Config, File};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Settings {
    pub port: u16,
    pub index: Index,
}

#[derive(Deserialize)]
pub struct Index {
    pub location: PathBuf,
}

pub fn load() -> Result<Settings> {
    let mut s = Config::new();

    s.merge(File::with_name("/app/crator.toml").required(false))?;
    s.merge(File::with_name("crator.toml").required(false))?;

    if let Ok(p) = std::env::var("PORT") {
        if let Ok(p) = p.parse::<i64>() {
            s.set("port", p)?;
        }
    }

    s.try_into().map_err(Into::into)
}
