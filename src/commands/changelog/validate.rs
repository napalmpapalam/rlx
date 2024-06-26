use clap::Args;
use keep_a_changelog::Changelog;
use serde::{Deserialize, Serialize};

use crate::{changelog_ext::ChangelogExt, context::Context, error::Result};

#[derive(Clone, Args, Debug, Serialize, Deserialize)]
pub(crate) struct ValidateCmd;

impl ValidateCmd {
    pub(crate) fn run(self, ctx: &Context) -> Result<()> {
        Changelog::from_ctx(ctx)?;
        ctx.success("Changelog is valid");
        Ok(())
    }
}
