use crate::error::Result;
use eyre::Context;
use serde::Deserialize;
use std::path::Path;

#[derive(Debug, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub workspace_path: Option<String>,
    pub debug: Option<bool>,
    pub changelog_path: Option<String>,
    pub remote_url: Option<String>,
    pub tag_prefix: Option<String>,
    pub head: Option<String>,
}

impl Config {
    pub fn new() -> Result<Self> {
        config::Config::builder()
            .add_source(config::File::from(Path::new(".rlx.yml")).required(false))
            .add_source(config::Environment::with_prefix("RLX"))
            .build()
            .wrap_err_with(|| "Failed to build config")?
            .try_deserialize()
            .wrap_err_with(|| "Failed to deserialize config")
            .map_err(Into::into)
    }
}
