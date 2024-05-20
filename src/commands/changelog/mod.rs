use clap::Subcommand;
use serde::{Deserialize, Serialize};

use crate::{context::Context, error::Result};

use self::{
    format::FormatCmd, get::GetCmd, new::NewCmd, release::ReleaseCmd, validate::ValidateCmd,
};

mod format;
mod get;
mod new;
mod release;
mod validate;

#[derive(Subcommand, Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) enum Changelog {
    /// Validate a changelog
    Validate(ValidateCmd),
    /// Get a release from a changelog
    Get(GetCmd),
    /// Make a release from [Unreleased] section
    Release(ReleaseCmd),
    /// Format a changelog
    Format(FormatCmd),
    /// Create a new changelog
    New(NewCmd),
}

impl Changelog {
    pub(super) async fn run(self, ctx: &Context) -> Result<()> {
        match self {
            Changelog::Validate(cmd) => cmd.run(ctx),
            Changelog::Get(cmd) => cmd.run(ctx),
            Changelog::Release(cmd) => cmd.run(ctx),
            Changelog::Format(cmd) => cmd.run(ctx),
            Changelog::New(cmd) => cmd.run(ctx),
        }
    }
}
