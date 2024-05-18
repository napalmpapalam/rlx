mod changelog_ext;
mod commands;
mod config;
mod context;
mod error;
mod log;
mod version_ext;

use clap::{
    builder::styling::{AnsiColor as Ansi, Styles},
    Parser,
};
use serde::{Deserialize, Serialize};

const STYLES: Styles = Styles::styled()
    .header(Ansi::Green.on_default().bold())
    .usage(Ansi::Green.on_default().bold())
    .literal(Ansi::BrightCyan.on_default().bold())
    .placeholder(Ansi::BrightCyan.on_default())
    .error(Ansi::Red.on_default().bold());

// TODO: Add proper printing error of parsing changelog
// TODO: Add apply-version command

#[derive(Clone, Parser, Debug, Serialize, Deserialize)]
#[clap(about)]
#[command(styles = STYLES)]
pub struct Opts {
    /// Path to the workspace directory with the packages directories if it's mono-repo (eg. "./packages").
    /// Used to infer the package(s) path for validating package.json version.
    ///
    /// If not provided, the current directory will be used.
    ///
    /// Can be set via `RLX_WORKSPACE_PATH` environment variable or `workspace_path` config option in the `.rlx.yml` file.
    #[arg(global = true, short, long)]
    pub workspace_path: Option<String>,
    /// Enable debug mode, which will print debug logs.
    ///
    /// Can be set via `RLX_DEBUG` environment variable or `debug` config option in the `.rlx.yml` file.
    #[arg(global = true, long)]
    pub debug: bool,
    /// The path to the changelog file, defaults to `CHANGELOG.md`
    ///
    /// Can be set via `RLX_CHANGELOG_PATH` environment variable or `changelog_path` config option in the `.rlx.yml` file.
    #[arg(global = true, alias = "cp", long)]
    pub changelog_path: Option<String>,
    /// The Git Remote URL of the repository, used to generate compare links in the changelog.
    ///
    /// If not provided it will be inferred from the git configuration.
    ///
    /// Can be set via `RLX_REMOTE_URL` environment variable or `remote_url` config option in the `.rlx.yml` file.
    #[arg(global = true, alias = "url", long)]
    #[serde(default)]
    pub remote_url: Option<String>,
    /// The tag prefix to use (e.g. `v`), used to generate compare links in the changelog.
    ///
    /// If not provided it will empty.
    ///
    /// Can be set via `RLX_TAG_PREFIX` environment variable or `tag_prefix` config option in the `.rlx.yml` file.
    #[arg(global = true, short, long)]
    #[serde(default)]
    pub tag_prefix: Option<String>,
    /// The head to use (by default `HEAD`), used to generate compare links in the changelog
    ///
    /// Can be set via `RLX_HEAD` environment variable or `head` config option in the `.rlx.yml` file.
    #[serde(default)]
    #[arg(global = true, long)]
    pub head: Option<String>,
    #[command(subcommand)]
    cmd: commands::Commands,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let opts: Opts = Opts::parse();
    let ctx = context::Context::new_from_options(&opts)?;
    if let Some(err) = opts.cmd.run(&ctx).await.err() {
        eprintln!("{}", err);
        std::process::exit(1);
    }

    Ok(())
}
