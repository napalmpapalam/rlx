use clap::Args;
use keep_a_changelog::{changelog::ChangelogBuilder, ReleaseBuilder};
use serde::{Deserialize, Serialize};

use crate::{
    context::Context,
    error::{Error, Result},
};

#[derive(Clone, Args, Debug, Serialize, Deserialize)]
pub(crate) struct NewCmd;

impl NewCmd {
    pub(crate) fn run(self, ctx: &Context) -> Result<()> {
        ctx.debug("Creating new changelog");

        let unreleased = ReleaseBuilder::default().build().map_err(Error::from)?;

        let changelog = ChangelogBuilder::default()
            .url(ctx.remote_url()?.to_owned())
            .tag_prefix(ctx.tag_prefix())
            .head(ctx.head())
            .releases(vec![unreleased])
            .build()
            .map_err(Error::from)?;

        ctx.debug("Saving new changelog");
        changelog.save_to_file(ctx.changelog_path())?;
        ctx.success("New changelog created successfully");

        Ok(())
    }
}
