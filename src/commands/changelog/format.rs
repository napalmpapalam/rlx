use clap::Args;
use eyre::Result;
use keep_a_changelog::Changelog;
use serde::{Deserialize, Serialize};

use crate::{changelog_ext::ChangelogExt, context::Context};

#[derive(Clone, Args, Debug, Serialize, Deserialize)]
pub(crate) struct FormatCmd;

impl FormatCmd {
    pub(crate) fn run(self, ctx: &Context) -> Result<()> {
        let changelog = Changelog::from_ctx(ctx)?;
        changelog.save_to_file(ctx.changelog_path())?;
        ctx.success("Changelog formatted successfully");
        Ok(())
    }
}
