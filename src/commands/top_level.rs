use clap::Subcommand;
use serde::{Deserialize, Serialize};

use crate::context::Context;

use super::*;

#[derive(Clone, Subcommand, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) enum Commands {
    /// Release Sanity Check
    #[command(name = "rsc")]
    ReleaseSanityCheck(rsc::ReleaseSanityCheck),
}

impl Commands {
    pub(crate) async fn run(self, context: &Context) -> anyhow::Result<()> {
        match self {
            Commands::ReleaseSanityCheck(cmd) => cmd.run(context).await,
        }
    }
}
