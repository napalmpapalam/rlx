use clap::Args;
use keep_a_changelog::Changelog;
use serde::{Deserialize, Serialize};

use crate::{changelog_ext::ChangelogExt, context::Context, error::Result};

#[derive(Clone, Args, Debug, Serialize, Deserialize)]
pub(crate) struct GetCmd {
    /// Get changes in a specific release. Use "latest" for the latest release
    version: String,
}

impl GetCmd {
    pub(crate) fn run(self, ctx: &Context) -> Result<()> {
        let version = self.version.replace('v', "");
        let changelog = Changelog::from_ctx(ctx)?;

        if version == "latest" {
            let release = changelog
                .releases()
                .iter()
                .find(|release| release.date().is_some() && release.version().is_some());

            if let Some(release) = release {
                println!("{}", release);
                return Ok(());
            }

            return Err("Latest release not found".into());
        }

        let release = changelog.find_release(self.version.clone())?;

        if let Some(release) = release {
            println!("{}", release);
            return Ok(());
        }

        Err(format!("{} release not found", self.version).into())
    }
}
