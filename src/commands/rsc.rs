use std::{fs::File, path::Path};

use colored::*;

use clap::Args;
use eyre::{eyre, Context as _Context, OptionExt, Result};
use serde::{Deserialize, Serialize};

use crate::context::Context;

pub struct PackageMetadata {
    version: String,
    name: String,
}

#[derive(Clone, Args, Debug, Serialize, Deserialize)]
pub struct ReleaseSanityCheck {
    /// The release version to check, if not provided, the not pushed git tag will be used.
    /// If no git tag is found, the check will be skipped.
    version: Option<String>,
}

impl ReleaseSanityCheck {
    pub async fn run(&self, ctx: &Context) -> Result<()> {
        let ver = self.get_release_version(ctx);
        if ver.is_none() {
            ctx.info("No version tag found, skipping release sanity check...");
            return Ok(());
        }

        let ver = ver.ok_or_eyre("Failed to get release version")?;
        ctx.debug(format!("Release package version: {ver}").as_str());
        ctx.debug("Validating semver compatibility of release");

        if !self.validate_semver_compatibility(ctx, ver.clone())? {
            return Ok(());
        }
        if !self.validate_package_version(ctx, ver.clone())? {
            ctx.error("Release package version check is failed");
            return Ok(());
        }
        if !self.validate_change_log(ctx, ver.clone())? {
            return Ok(());
        }
        ctx.success("Release changelog is valid");

        Ok(())
    }

    fn validate_change_log(&self, ctx: &Context, version: String) -> Result<bool> {
        let today_ymd = chrono::Local::now().format("%Y-%m-%d").to_string();
        let expected_title = format!("[{version}] - {today_ymd}");
        let err_msg = format!("\"## {expected_title}\" is absent in CHANGELOG.md");

        ctx.debug("Validating changelog");

        let latest_release = ctx
            .changelog()
            .releases()
            .iter()
            .find(|r| r.version().as_ref().map(|v| v.to_string()) == Some(version.clone()));
        if latest_release.is_none() {
            ctx.debug("Latest release not found in changelog");
            let link = "link here";
            ctx.error(format!("{err_msg} or \"{link}\" is absent in CHANGELOG.md").as_str());
            return Ok(false);
        }
        let latest_release = latest_release.expect("Failed to get latest release");

        let ver = latest_release.version().as_ref().map(|v| v.to_string());
        if ver.is_none() {
            ctx.debug("Version not found in latest release");
            ctx.error(err_msg.as_str());
            return Ok(false);
        }
        let ver = ver.expect("Failed to get latest release version");

        ctx.debug(format!("Latest release version: {ver}").as_str());

        if ver != version {
            ctx.debug("Latest release version is not equal to the release version");
            ctx.error(err_msg.as_str());
            return Ok(false);
        }

        let date = latest_release.date();
        if date.is_none() {
            ctx.debug("Date not found in latest release");
            ctx.error(err_msg.as_str());
            return Ok(false);
        }
        let date = date
            .expect("Failed to get latest release date")
            .format("%Y-%m-%d")
            .to_string();

        ctx.debug(format!("Latest release date: {date}").as_str());

        if date != today_ymd {
            ctx.debug("Latest release date is not equal to today's date");
            ctx.error(err_msg.as_str());
            return Ok(false);
        }

        let releases: Vec<String> = ctx
            .changelog()
            .releases()
            .iter()
            .filter(|r| r.version().is_some())
            .map(|r| {
                r.version()
                    .clone()
                    .expect("version should be present")
                    .to_string()
            })
            .collect();

        let repo_url = ctx.repository_url();
        let unreleased_url = get_git_compare_url(&repo_url, version.clone(), "HEAD".to_owned());
        let mut anchors = vec![fmt_anchor("Unreleased", unreleased_url.clone())];

        let mut invalid_anchors = !ctx.changelog_raw().contains(&unreleased_url);

        for (i, ver) in releases.iter().enumerate() {
            if i == releases.len() - 1 {
                let link = get_git_release_url(repo_url.clone(), ver.clone());
                if !ctx.changelog_raw().contains(&link) {
                    invalid_anchors = true
                }
                anchors.push(fmt_anchor(ver, link));
            } else {
                let previous = releases[i + 1_usize].clone();
                let link = get_git_compare_url(&repo_url, previous, ver.clone());
                if !ctx.changelog_raw().contains(&link) {
                    invalid_anchors = true
                }
                anchors.push(fmt_anchor(ver, link));
            }
        }

        if invalid_anchors {
            ctx.error_fmt(
                format!(
                    "{}\n{}",
                    "The anchors legend is invalid, should be: ".red().bold(),
                    anchors.join("\n").red()
                )
                .as_str(),
            );
        }
        Ok(!invalid_anchors)
    }

    fn validate_semver_compatibility(&self, ctx: &Context, version: String) -> Result<bool> {
        let semver = semver::Version::parse(&version);

        match semver {
            Ok(_) => {
                ctx.success("Release version is compatible with semantic versioning");
                Ok(true)
            }
            Err(e) => {
                ctx.error(
                    format!("Release version is not compatible with semantic versioning: {e}")
                        .as_str(),
                );
                Ok(false)
            }
        }
    }

    fn validate_package_version(&self, ctx: &Context, version: String) -> Result<bool> {
        if let Some(workspace_path) = ctx.workspace_path() {
            ctx.debug("Validating workspace package versions");
            return self.validate_workspace_package_versions(ctx, workspace_path, version);
        }

        ctx.debug("Validating single package version");
        self.validate_single_package_version(ctx, version, None)
    }

    fn validate_workspace_package_versions(
        &self,
        ctx: &Context,
        workspace_path: String,
        version: String,
    ) -> Result<bool> {
        let mut valid = true;
        let dir = Path::new(workspace_path.as_str());
        for entry in dir
            .read_dir()
            .wrap_err_with(|| "Failed to read workspace directory")?
        {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                let path_str = path
                    .to_str()
                    .ok_or(eyre!("Failed to convert path to string"))?
                    .to_string();

                valid = self
                    .validate_single_package_version(ctx, version.clone(), Some(path_str))
                    .wrap_err_with(|| "Failed to validate single package version during validating the workspace directory")?;
            }
        }

        Ok(valid)
    }

    fn validate_single_package_version(
        &self,
        ctx: &Context,
        release_version: String,
        dir: Option<String>,
    ) -> Result<bool> {
        let PackageMetadata { version, name } = self.get_package_metadata(ctx, dir)?;
        let valid = release_version == version;

        match valid {
            true => ctx.success_fmt(&format!(
                "{}{}{}",
                "Release version of the ".green(),
                name.clone().green().bold(),
                " is valid".green()
            )),
            false => ctx.error_fmt(&format!(
                "{}{}{}{}{}{}",
                "Release version of the ".red(),
                name.clone().red().bold(),
                " is invalid, expected: ".red(),
                release_version.clone().red().bold(),
                ", actual: ".red(),
                version.red().bold()
            )),
        }

        Ok(valid)
    }

    fn get_package_metadata(&self, ctx: &Context, dir: Option<String>) -> Result<PackageMetadata> {
        let path = dir.unwrap_or_else(|| ".".to_string()) + "/package.json";

        ctx.debug(format!("Reading package.json file from path {path}").as_str());

        let file = File::open(
            Path::new(path.as_str())
                .canonicalize()
                .wrap_err_with(|| "Failed to build package.json file path")?,
        )
        .wrap_err_with(|| "Failed to open package.json file")?;
        let json: serde_json::Value = serde_json::from_reader(file)?;
        let version = json
            .get("version")
            .and_then(|v| v.as_str())
            .map(|v| v.to_string())
            .ok_or_eyre("No version found in package.json")?;
        let name = json
            .get("name")
            .and_then(|v| v.as_str())
            .map(|v| v.to_string())
            .ok_or_eyre("No name found in package.json")?;

        Ok(PackageMetadata { version, name })
    }

    fn get_release_version(&self, ctx: &Context) -> Option<String> {
        let git_tag = ctx.git_tag();
        self.version.clone().or(git_tag)
    }
}

fn get_git_release_url(repo_url: String, version: String) -> String {
    let mut url_body = "/-/tags/";
    if repo_url.starts_with("https://github.com") {
        url_body = "/releases/tag/";
    }

    format!("{repo_url}{url_body}{version}")
}

fn get_git_compare_url(repo_url: &str, previous: String, current: String) -> String {
    format!("{repo_url}/compare/{previous}...{current}")
}

fn fmt_anchor(version: &str, link: String) -> String {
    format!("[{version}]: {link}")
}
