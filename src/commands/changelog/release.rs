use chrono::Local;
use clap::Args;
use eyre::Result;
use keep_a_changelog::{Changelog, Release};
use semver::Version;
use serde::{Deserialize, Serialize};

use crate::{changelog_ext::ChangelogExt, context::Context};

#[derive(Clone, Args, Debug, Serialize, Deserialize)]
pub(crate) struct ReleaseCmd {
    /// Release version
    version: String,
}

impl ReleaseCmd {
    pub(crate) fn run(self, ctx: &Context) -> Result<()> {
        let version: Version = self.version.parse()?;
        let mut changelog = Changelog::from_ctx(ctx)?;
        let unreleased = changelog.get_unreleased_mut();

        if unreleased.is_none() {
            ctx.error("Unreleased section not found");
            return Ok(());
        }

        let unreleased = unreleased.unwrap();

        if unreleased.changes().is_empty() {
            ctx.error("No changes found in the unreleased section");
            return Ok(());
        }

        let release = Release::builder()
            .version(version.clone())
            .date(Local::now().naive_local())
            .changes(unreleased.changes().clone())
            .build()?;

        unreleased.empty_changes();
        changelog.add_release(release);
        changelog.save_to_file(ctx.changelog_path())?;

        ctx.success(&format!("Release [{}] added", version));

        Ok(())
    }
}
