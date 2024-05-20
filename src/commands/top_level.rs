use clap::Subcommand;
use serde::{Deserialize, Serialize};

use crate::{
    context::Context,
    error::{Error, Result},
};

use super::*;

#[derive(Clone, Subcommand, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Commands {
    /// Release Sanity Check.
    /// Check that a release is sane (`package.json`, `CHANGELOG.md` and semantic versioning are
    /// valid for the release)
    #[command(name = "rsc")]
    ReleaseSanityCheck(rsc::ReleaseSanityCheck),
    /// Changelog commands, used to parse and manipulate changelog
    #[command(alias = "cl")]
    Changelog {
        #[command(subcommand)]
        #[serde(flatten)]
        cmd: changelog::Changelog,
    },

    /// Version commands, used to manipulate versions
    #[command(alias = "v")]
    Version {
        #[command(subcommand)]
        #[serde(flatten)]
        cmd: version::Version,
    },
}

impl Commands {
    pub async fn run(self, context: &Context) -> Result<()> {
        match self {
            Commands::ReleaseSanityCheck(cmd) => cmd.run(context).await.map_err(Error::from),
            Commands::Version { cmd } => cmd.run(context).await.map_err(Error::from),
            Commands::Changelog { cmd } => cmd.run(context).await.map_err(Error::from),
        }
    }
}
