use std::{fs::File, path::Path};

use colored::*;

use clap::Args;
use eyre::{Context as _Context, OptionExt};
use keep_a_changelog::{Changelog, ChangelogParseOptions};
use serde::{Deserialize, Serialize};

use crate::{
    context::Context,
    error::{Error, Result},
};

pub struct PackageMetadata {
    version: String,
    name: String,
}

#[derive(Clone, Args, Debug, Serialize, Deserialize)]
pub struct ReleaseSanityCheck {
    /// The release version to check, if not provided, the not pushed git tag will be used.
    /// If no git tag is found, the check will be skipped.
    version: Option<String>,
    /// The prefix of the git tags to use for the changelog link validation.
    /// If not provided, the default value will be empty.
    #[arg(short, long)]
    tag_prefix: Option<String>,
    /// The git ref to use as the head for the changelog link validation.
    /// If not provided, the default value will be `HEAD`.
    #[arg(short, long)]
    head: Option<String>,
}

impl ReleaseSanityCheck {
    pub async fn run(&self, ctx: &Context) -> Result<()> {
        let ver = self.get_release_version(ctx)?;
        if ver.is_none() {
            ctx.info("No version tag found, skipping release sanity check...");
            return Ok(());
        }

        let ver = ver.ok_or_eyre("Failed to get release version")?;
        ctx.debug(format!("Release package version: {ver}").as_str());
        ctx.debug("Validating semver compatibility of release");

        self.validate_semver_compatibility(ctx, ver.clone())?;

        ctx.debug("Validating package(s) version(s)");

        self.validate_package_version(ctx, ver.clone())?;
        self.validate_change_log(ctx, ver.clone())?;

        ctx.success("Release changelog is valid");
        Ok(())
    }

    fn validate_change_log(&self, ctx: &Context, version: String) -> Result<()> {
        let today_ymd = chrono::Local::now().format("%Y-%m-%d").to_string();
        let expected_title = format!("[{version}] - {today_ymd}");
        let err_msg = format!("\"## {expected_title}\" is absent in CHANGELOG.md");
        let repo_url = ctx.remote_url()?.to_owned();

        ctx.debug("Validating changelog");

        let changelog = Changelog::parse_from_file(
            "CHANGELOG.md",
            Some(ChangelogParseOptions {
                url: Some(repo_url.clone()),
                tag_prefix: self.tag_prefix.clone(),
                head: self.head.clone(),
            }),
        )?;

        let latest = changelog
            .releases()
            .iter()
            .find(|r| r.version().is_some())
            .ok_or_else(|| Error::from(err_msg.clone()))?;
        let latest_version = latest
            .version()
            .as_ref()
            .map(|v| v.to_string())
            .ok_or_eyre("Failed to get latest release version")?;

        ctx.debug(format!("Latest release version: {latest_version}").as_str());

        if latest_version != version {
            ctx.debug("Latest release version is not equal to the release version");
            return Err(err_msg.into());
        }

        let latest_date = latest
            .date()
            .ok_or_eyre("Failed to get latest release date")?
            .format("%Y-%m-%d")
            .to_string();

        ctx.debug(format!("Latest release date: {latest_date}").as_str());

        if latest_date != today_ymd {
            ctx.debug("Latest release date is not equal to today's date");
            return Err(err_msg.into());
        }

        let links = changelog
            .links()
            .iter()
            .filter(|l| l.url().contains(&repo_url))
            .collect::<Vec<_>>();

        let mut anchors = vec![];
        let mut invalid_anchors = false;

        for release in changelog.releases() {
            let link = release
                .compare_link(&changelog)?
                .ok_or_eyre("Failed to get compare link")?;

            let release_version = release.version();

            if let Some(version) = release_version {
                anchors.push(fmt_anchor(&version.to_string(), link.url.clone()));
            } else {
                anchors.push(fmt_anchor("Unreleased", link.url.clone()));
            }

            if !invalid_anchors {
                invalid_anchors = !links.iter().any(|l| {
                    let url_match = *l.url() == link.url;
                    let anchor = l.anchor();
                    let anchor_match = match release_version {
                        Some(version) => l.anchor() == &version.to_string(),
                        None => anchor == "Unreleased",
                    };

                    url_match && anchor_match
                });
            }
        }

        if invalid_anchors {
            return Err(Error::new_fmt(format!(
                "{}\n{}",
                "The anchors legend is invalid, should be: ".red().bold(),
                anchors.join("\n").red()
            )));
        }

        Ok(())
    }

    fn validate_semver_compatibility(&self, ctx: &Context, version: String) -> Result<()> {
        semver::Version::parse(&version).map_err(|e| {
            format!("Release version is not compatible with semantic versioning: {e}")
        })?;

        ctx.success("Release version is compatible with semantic versioning");
        Ok(())
    }

    fn validate_package_version(&self, ctx: &Context, version: String) -> Result<()> {
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
    ) -> Result<()> {
        let dir = Path::new(workspace_path.as_str());
        for entry in dir
            .read_dir()
            .wrap_err_with(|| "Failed to read workspace directory")?
        {
            let entry = entry.wrap_err_with(|| "Failed to read package directory")?;
            let path = entry.path();
            if path.is_dir() {
                let path_str = path
                    .to_str()
                    .ok_or_eyre("Failed to convert path to string")?
                    .to_string();

                self.validate_single_package_version(ctx, version.clone(), Some(path_str))?;
            }
        }

        Ok(())
    }

    fn validate_single_package_version(
        &self,
        ctx: &Context,
        release_version: String,
        dir: Option<String>,
    ) -> Result<()> {
        let PackageMetadata { version, name } = self.get_package_metadata(ctx, dir)?;
        let valid = release_version == version;

        if !valid {
            return Err(Error::new_fmt(format!(
                "{}{}{}{}{}{}",
                "Release version of the ".red(),
                name.clone().red().bold(),
                " is invalid, expected: ".red(),
                release_version.clone().red().bold(),
                ", actual: ".red(),
                version.red().bold()
            )));
        }

        ctx.success_fmt(&format!(
            "{}{}{}",
            "Release version of the ".green(),
            name.clone().green().bold(),
            " is valid".green()
        ));
        Ok(())
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
        let json: serde_json::Value =
            serde_json::from_reader(file).wrap_err_with(|| "Failed to create json from reader")?;
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

    fn get_release_version(&self, ctx: &Context) -> Result<Option<String>> {
        let git_tag = ctx.git_tag()?.to_owned();
        Ok(self.version.clone().or(git_tag))
    }
}

fn fmt_anchor(version: &str, link: String) -> String {
    format!("[{version}]: {link}")
}
