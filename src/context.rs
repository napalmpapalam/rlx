use eyre::{Context as _Context, OptionExt, Result};
use git2::Repository;
use regex::Regex;

use crate::config;

pub struct Context {
    workspace_path: Option<String>,
    debug: bool,
}

impl Context {
    pub fn new_from_options(options: &super::Opts) -> Result<Self> {
        let config = config::Config::new()
            .wrap_err_with(|| "Failed to load config")
            .ok();

        let workspace_path = options
            .workspace_path
            .clone()
            .or_else(|| config.as_ref().and_then(|c| c.workspace_path.clone()));

        let debug = options
            .debug
            .or_else(|| config.as_ref().and_then(|c| c.debug))
            .unwrap_or(false);

        Ok(Self {
            workspace_path,
            debug,
        })
    }

    pub fn repository(&self) -> Result<String> {
        let repo = Repository::open(std::env::current_dir()?)?;
        let origin = repo
            .find_remote("origin")
            .wrap_err_with(|| "Failed to find origin remote")?;
        let url = origin.url().ok_or_eyre("Failed to get git remote URL")?;
        normalize_origin_url(url)
    }

    pub fn git_tag(&self) -> Result<Option<String>> {
        let repo = Repository::open(std::env::current_dir()?)?;

        let tag = repo
            .tag_names(None)?
            .iter()
            .next()
            .unwrap_or(None)
            .map(|t| t.to_string());

        Ok(tag)
    }

    pub fn workspace_path(&self) -> Option<String> {
        self.workspace_path.clone()
    }

    pub fn debug(&self) -> bool {
        self.debug
    }
}

fn normalize_origin_url(url: &str) -> Result<String> {
    let rx = Regex::new(r"git@(.+):(.+)\.git")?;
    Ok(rx.replace(url, "https://$1/$2").to_string())
}
