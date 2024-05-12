use std::process::Command;

use eyre::{bail, Context as _Context, OptionExt, Result};
use git2::Repository;
use once_cell::sync::OnceCell;
use regex::Regex;

use crate::{config::Config, log::Logger};

pub struct Context {
    cfg: Config,
    log: Logger,
    head: String,
    git_tag: OnceCell<Option<String>>,
    tag_prefix: Option<String>,
    remote_url: OnceCell<String>,
    workspace_path: Option<String>,
    changelog_path: String,
}

impl Context {
    pub fn new_from_options(options: &super::Opts) -> Result<Self> {
        let cfg = Config::new().wrap_err_with(|| "Failed to load config")?;

        let workspace_path = options
            .workspace_path
            .clone()
            .or_else(|| cfg.workspace_path.clone());

        let debug = options.debug || cfg.debug.unwrap_or_default();

        let head = options
            .head
            .clone()
            .or_else(|| cfg.head.clone())
            .unwrap_or_else(|| "HEAD".to_owned());

        let changelog_path = options
            .changelog_path
            .clone()
            .or_else(|| cfg.changelog_path.clone())
            .unwrap_or_else(|| "CHANGELOG.md".to_owned());

        let tag_prefix = options
            .tag_prefix
            .clone()
            .or_else(|| cfg.tag_prefix.clone());

        Ok(Self {
            cfg,
            head,
            tag_prefix,
            workspace_path,
            changelog_path,
            log: Logger::new(debug),
            git_tag: OnceCell::new(),
            remote_url: OnceCell::new(),
        })
    }

    pub fn workspace_path(&self) -> Option<String> {
        self.workspace_path.clone()
    }

    pub fn head(&self) -> String {
        self.head.clone()
    }

    pub fn changelog_path(&self) -> &str {
        &self.changelog_path
    }

    pub fn tag_prefix(&self) -> Option<String> {
        self.tag_prefix.clone()
    }

    pub fn remote_url(&self) -> Result<&String> {
        self.remote_url.get_or_try_init(|| {
            if let Some(url) = &self.cfg.remote_url {
                return normalize_origin_url(url.as_str());
            }

            let repo = Repository::open(std::env::current_dir()?)?;
            let origin = repo
                .find_remote("origin")
                .wrap_err_with(|| "Failed to find origin remote")?;
            let url = origin.url().ok_or_eyre("Failed to get git remote URL")?;
            normalize_origin_url(url)
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

    #[allow(dead_code)]
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
