use std::process::Command;

use eyre::{bail, Context as _Context, OptionExt, Result};
use git2::Repository;
use once_cell::sync::OnceCell;
use regex::Regex;

use crate::{config, log::Logger};

pub struct Context {
    workspace_path: Option<String>,
    repository_url: OnceCell<String>,
    git_tag: OnceCell<Option<String>>,
    log: Logger,
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

        let debug = options.debug || config.as_ref().and_then(|c| c.debug).unwrap_or(false);

        Ok(Self {
            log: Logger::new(debug),
            git_tag: OnceCell::new(),
            repository_url: OnceCell::new(),
            workspace_path,
        })
    }

    pub fn workspace_path(&self) -> Option<String> {
        self.workspace_path.clone()
    }

    pub fn repository_url(&self) -> Result<&String> {
        self.repository_url.get_or_try_init(|| {
            let repo = Repository::open(std::env::current_dir()?)?;
            let origin = repo
                .find_remote("origin")
                .wrap_err_with(|| "Failed to find origin remote")?;
            let repository_url = origin.url().ok_or_eyre("Failed to get git remote URL")?;
            let repository_url = normalize_origin_url(repository_url)?;
            Ok(repository_url)
        })
    }

    pub fn git_tag(&self) -> Result<&Option<String>> {
        self.git_tag.get_or_try_init(|| {
            let output = Command::new("git")
                .arg("log")
                .arg("-1")
                .arg("--format=\"%D\"")
                .output()?;

            if !output.status.success() {
                bail!("Git command executed with failing error code");
            }

            let refs_report = String::from_utf8_lossy(&output.stdout);
            let rx = Regex::new(r"/tag: ([\w\d\-_.]+)/i")?;
            let version_match = rx.captures(&refs_report);

            if version_match.is_none() {
                return Ok(None);
            }

            Ok(version_match
                .ok_or_eyre("Failed to get version from git tag")?
                .get(1)
                .map(|m| m.as_str().to_string()))
        })
    }

    pub fn error(&self, msg: &str) {
        self.log.error(msg);
    }

    pub fn error_fmt(&self, msg: &str) {
        self.log.error_fmt(msg);
    }

    pub fn info(&self, msg: &str) {
        self.log.info(msg);
    }

    pub fn success(&self, msg: &str) {
        self.log.success(msg);
    }

    pub fn success_fmt(&self, msg: &str) {
        self.log.success_fmt(msg);
    }

    pub fn debug(&self, msg: &str) {
        self.log.debug(msg);
    }
}

fn normalize_origin_url(url: &str) -> Result<String> {
    let rx = Regex::new(r"git@(.+):(.+)\.git")?;
    Ok(rx.replace(url, "https://$1/$2").to_string())
}
