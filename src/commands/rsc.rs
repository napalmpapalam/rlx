use std::{fs::File, path::Path};

use clap::Args;
use eyre::{eyre, Context as _Context, OptionExt, Result};
use serde::{Deserialize, Serialize};

use crate::{
    context::Context,
    print::{
        debug, error, print_checking_versions, print_invalid_package_version,
        print_valid_package_version,
    },
};

pub(crate) struct PackageMetadata {
    version: String,
    name: String,
}

#[derive(Clone, Args, Debug, Serialize, Deserialize)]
/// Check that a release is sane (package.json, CHANGELOG.md, etc.)
pub(crate) struct ReleaseSanityCheck {
    /// The release version to check
    version: String,
}

impl ReleaseSanityCheck {
    pub(crate) async fn run(&self, ctx: &Context) -> Result<()> {
        print_checking_versions();
        let release_version = self.version.clone();
        debug(
            ctx,
            format!("Provided release package version: {release_version}").as_str(),
        );
        let valid = self.validate_package_version(ctx, self.version.clone())?;

        if !valid {
            error("Package version check is failed");
            return Ok(());
        }

        println!("Repository URL: {}", ctx.repository()?);
        Ok(())
    }

    fn validate_package_version(&self, ctx: &Context, version: String) -> Result<bool> {
        if let Some(workspace_path) = ctx.workspace_path() {
            debug(ctx, "Validating workspace package versions");
            return self.validate_workspace_package_versions(ctx, workspace_path, version);
        }

        debug(ctx, "Validating single package version");
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
        for entry in dir.read_dir()? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                let path_str = path
                    .to_str()
                    .ok_or(eyre!("Failed to convert path to string"))?
                    .to_string();

                valid =
                    self.validate_single_package_version(ctx, version.clone(), Some(path_str))?;
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
            true => print_valid_package_version(name.clone()),
            false => print_invalid_package_version(name.clone(), release_version.clone(), version),
        }

        Ok(valid)
    }

    fn get_package_metadata(&self, ctx: &Context, dir: Option<String>) -> Result<PackageMetadata> {
        let path = dir.unwrap_or(".".to_string()) + "/package.json";

        debug(
            ctx,
            format!("Reading package.json file from path {path}").as_str(),
        );

        let file = File::open(
            Path::new(path.as_str())
                .canonicalize()
                .with_context(|| "Failed to build package.json file path")?,
        )
        .with_context(|| "Failed to open package.json file")?;
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
}
