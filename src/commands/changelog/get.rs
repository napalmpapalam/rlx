use clap::Args;
use eyre::Result;
use keep_a_changelog::Changelog;
use serde::{Deserialize, Serialize};

use crate::{changelog_ext::ChangelogExt, context::Context};

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

            ctx.error("Latest release not found");
            return Ok(());
        }

        let release = changelog.find_release(self.version.clone())?;

        if let Some(release) = release {
            println!("{}", release);
            return Ok(());
        }

        ctx.error(&format!("{} release not found", self.version));
        Ok(())
    }
}
