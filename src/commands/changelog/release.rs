use chrono::Local;
use clap::Args;
use eyre::{eyre, OptionExt};
use keep_a_changelog::{Changelog, Release};
use semver::Version;
use serde::{Deserialize, Serialize};

use crate::{changelog_ext::ChangelogExt, context::Context, error::Result};

#[derive(Clone, Args, Debug, Serialize, Deserialize)]
pub(crate) struct ReleaseCmd {
    /// Release version
    version: String,
}

impl ReleaseCmd {
    pub(crate) fn run(self, ctx: &Context) -> Result<()> {
        let version: Version = self
            .version
            .parse()
            .map_err(|e| eyre!("Failed to parse version: {e}"))?;
        let mut changelog = Changelog::from_ctx(ctx)?;
        let unreleased = changelog
            .get_unreleased_mut()
            .ok_or_eyre("Unreleased section not found")?;

        if unreleased.changes().is_empty() {
            return Err("No changes found in the unreleased section".into());
        }

        let release = Release::builder()
            .version(version.clone())
            .date(Local::now().naive_local())
            .changes(unreleased.changes().clone())
            .build()
            .map_err(|e| eyre!("Failed to build release: {e}"))?;

        unreleased.empty_changes();
        changelog.add_release(release);
        changelog.save_to_file(ctx.changelog_path())?;

        ctx.success(&format!("Release [{}] added", version));

        Ok(())
    }
}
