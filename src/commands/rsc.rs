use std::{
    fs::{self, File},
    path::Path,
};

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
        let ver = self.get_release_version(ctx)?;
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

        println!("Repository URL: {}", ctx.repository()?);
        Ok(())
    }

    fn validate_change_log(&self, ctx: &Context, version: String) -> Result<bool> {
        let raw = fs::read_to_string(Path::new("CHANGELOG.md"))
            .wrap_err_with(|| "Failed to read raw changelog file")?;
        let changelog =
            parse_changelog::parse(&raw).wrap_err_with(|| "Failed to parse changelog file")?;

        let mut latest_release = changelog[0].clone();
        if latest_release.title.starts_with("[Unreleased]") {
            latest_release = changelog[1].clone();
        }

        let today_ymd = chrono::Local::now().format("%Y-%m-%d").to_string();

        let expected_title = format!("[{version}] - {today_ymd}");

        if latest_release.title != expected_title {
            ctx.error(format!("\"## {expected_title}\" is absent in CHANGELOG.md").as_str());
            return Ok(false);
        }

        Ok(true)
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
        let path = dir.unwrap_or(".".to_string()) + "/package.json";

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

    fn get_release_version(&self, ctx: &Context) -> Result<Option<String>> {
        let git_tag = ctx
            .git_tag()
            .wrap_err_with(|| "Failed to get latest git tag")?;
        Ok(self.version.clone().or(git_tag))
    }
}
