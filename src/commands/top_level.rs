use clap::Subcommand;
use eyre::Result;
use serde::{Deserialize, Serialize};

use crate::context::Context;

use super::*;

#[derive(Clone, Subcommand, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Commands {
    /// Release Sanity Check.
    /// Check that a release is sane (`package.json`, `CHANGELOG.md` and semantic versioning are
    /// valid for the release)
    #[command(name = "rsc")]
    ReleaseSanityCheck(rsc::ReleaseSanityCheck),
    /// Change Log commands, used to parse and manipulate changelog
    #[command(alias = "cl")]
    ChangeLog {
        #[command(subcommand)]
        #[serde(flatten)]
        cmd: changelog::Changelog,
    },
}

impl Commands {
    pub async fn run(self, context: &Context) -> Result<()> {
        match self {
            Commands::ReleaseSanityCheck(cmd) => cmd.run(context).await,
            Commands::ChangeLog { cmd } => cmd.run(context).await,
        }
    }
}
