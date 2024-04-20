use url::Url;

use crate::config;

pub(crate) struct Context {
    repository: Option<Url>,
}

impl Context {
    pub(crate) fn new_from_options(options: &super::Opts) -> anyhow::Result<Self> {
        let config = config::Config::new()
            .map_err(|err| {
                println!("{err:?}");
                err
            })
            .ok();

        let repository = options
            .repository
            .clone()
            .or_else(|| config.as_ref().and_then(|c| c.repository.clone()));

        Ok(Self { repository })
    }

    pub(crate) fn repository(&self) -> anyhow::Result<Url> {
        self.repository
            .clone()
            .ok_or_else(|| anyhow::anyhow!("Git Repository URL missing"))
    }
}
