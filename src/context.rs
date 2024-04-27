use std::{fs::File, io::Read, path::Path, process::Command};

use clparse::{changelog::Changelog, ChangelogParser};
use eyre::{eyre, Context as _Context, OptionExt, Result};
use git2::Repository;
use keep_a_changelog::{changelog::ChangeLogParseOptions, Changelog as MyChangelog};
use regex::Regex;

use crate::{config, log::Logger};

pub struct Context {
    workspace_path: Option<String>,
    repository_url: String,
    git_tag: Option<String>,
    changelog: Changelog,
    changelog_raw: String,
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

        let repo = Repository::open(std::env::current_dir()?)?;
        let origin = repo
            .find_remote("origin")
            .wrap_err_with(|| "Failed to find origin remote")?;
        let repository_url = origin.url().ok_or_eyre("Failed to get git remote URL")?;
        let repository_url = normalize_origin_url(repository_url)?;

        let path = Path::new("CHANGELOG.md");
        let changelog = ChangelogParser::new("-".to_owned(), None)
            .parse(path.to_path_buf())
            .map_err(|e| eyre!(e.to_string()))
            .wrap_err_with(|| "Failed to parse changelog")?;

        let mut changelog_raw = String::new();
        File::open(path)?.read_to_string(&mut changelog_raw)?;
        let changelog_raw = changelog_raw;

        // TODO: Add ability to provide tag name prefix
        let cl = MyChangelog::parse(
            "CHANGELOG.md",
            ChangeLogParseOptions {
                url: Some(repository_url.clone()),
                ..Default::default()
            },
        )?;

        println!("{cl}");

        Ok(Self {
            log: Logger::new(debug),
            git_tag: get_git_tag()?,
            changelog,
            changelog_raw,
            workspace_path,
            repository_url,
        })
    }

    pub fn workspace_path(&self) -> Option<String> {
        self.workspace_path.clone()
    }

    pub fn repository_url(&self) -> String {
        self.repository_url.clone()
    }

    pub fn git_tag(&self) -> Option<String> {
        self.git_tag.clone()
    }

    pub fn changelog(&self) -> &Changelog {
        &self.changelog
    }

    pub fn changelog_raw(&self) -> &str {
        &self.changelog_raw
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

fn get_git_tag() -> Result<Option<String>> {
    let output = Command::new("git")
        .arg("log")
        .arg("-1")
        .arg("--format=\"%D\"")
        .output()?;

    if !output.status.success() {
        return Err(eyre!("Git command executed with failing error code"));
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
}
