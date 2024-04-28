use clap::Subcommand;
use eyre::Result;
use keep_a_changelog::ChangelogParseOptions;
use serde::{Deserialize, Serialize};

use crate::context::Context;

#[derive(Subcommand, Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) enum ChangeLog {
    /// Parse a changelog and print it to the console
    Parse {
        /// The path to the changelog file
        path: String,
        /// The URL to the repository, used to generate compare links in the changelog,
        /// if not provided it will be inferred from the git configuration
        #[arg(short, long)]
        #[serde(default)]
        url: Option<String>,
        /// The tag prefix to use (e.g. `v`), used to generate compare links in the changelog
        #[arg(short, long)]
        #[serde(default)]
        tag_prefix: Option<String>,
        /// The head to use (by default `HEAD`), used to generate compare links in the changelog
        #[serde(default)]
        #[arg(short, long, default_value = "HEAD")]
        head: Option<String>,
    },
}

impl ChangeLog {
    pub(super) async fn run(self, _context: &Context) -> Result<()> {
        match self {
            ChangeLog::Parse {
                path,
                url,
                tag_prefix,
                head,
            } => {
                let changelog = keep_a_changelog::Changelog::parse_from_file(
                    &path,
                    Some(ChangelogParseOptions {
                        url,
                        tag_prefix,
                        head,
                    }),
                )?;
                println!("{changelog}");
                Ok(())
            }
        }
    }
}
