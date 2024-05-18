use clap::Subcommand;
use serde::{Deserialize, Serialize};

use crate::{context::Context, error::Result};

use self::apply::ApplyCmd;

mod apply;

#[derive(Subcommand, Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) enum Version {
    /// Apply a version to a package.json
    Apply(ApplyCmd),
}

impl Version {
    pub(super) async fn run(self, ctx: &Context) -> Result<()> {
        match self {
            Version::Apply(cmd) => cmd.run(ctx),
        }
    }
}
