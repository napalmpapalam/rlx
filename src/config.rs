use eyre::{Context, Result};
use serde::Deserialize;
use std::path::Path;

#[derive(Debug, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub workspace_path: Option<String>,
    pub debug: Option<bool>,
}

impl Config {
    pub fn new() -> Result<Self> {
        Ok(config::Config::builder()
            .add_source(config::File::from(Path::new(".rlx.yml")).required(false))
            .add_source(config::Environment::with_prefix("RLX"))
            .build()
            .with_context(|| "Failed to build config")?
            .try_deserialize()?)
    }
}
