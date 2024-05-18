use std::{
    fs::{File, OpenOptions},
    io::Write,
    path::Path,
};

use crate::{context::Context, error::Result};
use clap::Args;
use colored::Colorize;
use eyre::{eyre, OptionExt};
use serde::{Deserialize, Serialize};

#[derive(Clone, Args, Debug, Serialize, Deserialize)]
pub(crate) struct ApplyCmd {
    /// The version to apply to the package.json
    version: String,
}

impl ApplyCmd {
    pub(crate) fn run(self, ctx: &Context) -> Result<()> {
        let version = self.version.clone();

        self.validate_semver_compatibility(version.clone())?;

        if let Some(workspace_path) = ctx.workspace_path() {
            ctx.debug("Appling version to workspace packages");
            return self.apply_workspace_versions(ctx, workspace_path, version);
        }

        ctx.debug("Appling single package version");
        self.apply_version(ctx, version, None)
    }

    fn validate_semver_compatibility(&self, version: String) -> Result<()> {
        semver::Version::parse(&version)
            .map_err(|e| format!("Version is not compatible with semantic versioning: {e}"))?;

        Ok(())
    }

    fn apply_version(&self, ctx: &Context, version: String, dir: Option<String>) -> Result<()> {
        let path = dir.unwrap_or_else(|| ".".to_string()) + "/package.json";

        ctx.debug(format!("Reading package.json file from path {path}").as_str());

        let file = File::open(
            Path::new(path.as_str())
                .canonicalize()
                .map_err(|e| eyre!("Failed to build package.json file path: {e}"))?,
        )
        .map_err(|e| eyre!("Failed to open package.json file: {e}"))?;

        let mut json: serde_json::Value = serde_json::from_reader(file)
            .map_err(|e| eyre!("Failed to create json from reader: {e}"))?;

        let package_version = json
            .get_mut("version")
            .ok_or_else(|| eyre!("Version field not found in package.json by path: \"{path}\"",))?;

        *package_version = serde_json::Value::String(version.to_string());

        let name = json
            .get("name")
            .ok_or_else(|| eyre!("Name field not found in package.json by path: \"{path}\"",))?;

        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(path)
            .map_err(|e| eyre!("Failed to open package.json file: {e}"))?;
        file.write_all(
            format!(
                "{}\n",
                serde_json::to_string_pretty(&json).expect("Expect package.json valid")
            )
            .as_bytes(),
        )
        .map_err(|e| eyre!("Failed to write to package.json file: {e}"))?;
        file.flush()
            .map_err(|e| eyre!("Failed to flush package.json write output: {e}"))?;

        ctx.success_fmt(&format!(
            "{}: {} {} {}",
            format!("[{name}]").as_str().yellow().bold(),
            "version".green(),
            version.bold().green(),
            "has been applied".green()
        ));

        Ok(())
    }

    fn apply_workspace_versions(
        &self,
        ctx: &Context,
        workspace_path: String,
        version: String,
    ) -> Result<()> {
        let dir = Path::new(workspace_path.as_str());
        for entry in dir
            .read_dir()
            .map_err(|e| eyre!("Failed to read workspace directory: {e}"))?
        {
            let entry = entry.map_err(|e| eyre!("Failed to read package directory: {e}"))?;
            let path = entry.path();
            if path.is_dir() {
                let path_str = path
                    .to_str()
                    .ok_or_eyre("Failed to convert path to string")?
                    .to_string();

                self.apply_version(ctx, version.clone(), Some(path_str))?;
            }
        }

        Ok(())
    }
}
