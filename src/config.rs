use std::path::Path;

pub use config::ConfigError;
use serde::Deserialize;
use url::Url;

#[derive(Debug, Deserialize)]
pub(crate) struct Config {
    #[serde(default)]
    pub(crate) repository: Option<Url>,
}

impl Config {
    pub fn new() -> Result<Self, ConfigError> {
        config::Config::builder()
            .add_source(config::File::from(Path::new(".rlx.yml")).required(false))
            .add_source(config::Environment::with_prefix("RLX"))
            .build()?
            .try_deserialize()
    }
}
