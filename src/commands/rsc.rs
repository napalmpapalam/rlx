use std::path::Path;

use clap::Args;
use package_json::PackageJsonManager;
use serde::{Deserialize, Serialize};

use crate::context::Context;

#[derive(Clone, Args, Debug, Serialize, Deserialize)]
/// Check that a release is sane (package.json, CHANGELOG.md, etc.)
pub(crate) struct ReleaseSanityCheck {
    /// The release version to check
    version: String,
}

impl ReleaseSanityCheck {
    pub(crate) async fn run(&self, ctx: &Context) -> anyhow::Result<()> {
        println!("Provided version: {}", self.version);
        println!("Package version: {}", self.get_package_version()?);
        println!("Repository URL: {}", ctx.repository()?);
        Ok(())
    }

    fn get_package_version(&self) -> anyhow::Result<String> {
        let mut manager = PackageJsonManager::new();
        // based on the given path
        manager.locate_closest_from(Path::new("package.json"))?;
        let json = manager.read_ref()?;
        Ok(json.version.clone())
    }
}
