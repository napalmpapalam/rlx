use clap::Subcommand;
use eyre::Result;
use serde::{Deserialize, Serialize};

use crate::context::Context;

use super::*;

#[derive(Clone, Subcommand, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Commands {
    /// Release Sanity Check
    #[command(name = "rsc")]
    ReleaseSanityCheck(rsc::ReleaseSanityCheck),
}

impl Commands {
    pub async fn run(self, context: &Context) -> Result<()> {
        match self {
            Commands::ReleaseSanityCheck(cmd) => cmd.run(context).await,
        }
    }
}
