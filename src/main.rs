mod commands;
mod config;
mod context;
mod log;

use clap::{
    builder::styling::{AnsiColor as Ansi, Styles},
    Parser,
};

const STYLES: Styles = Styles::styled()
    .header(Ansi::Green.on_default().bold())
    .usage(Ansi::Green.on_default().bold())
    .literal(Ansi::BrightCyan.on_default().bold())
    .placeholder(Ansi::BrightCyan.on_default())
    .error(Ansi::Red.on_default().bold());

#[derive(Clone, Parser, Debug)]
#[clap(about)]
#[command(styles = STYLES)]
pub struct Opts {
    /// Path to the workspace directory with the packages directories if it's mono-repo (eg. "./packages").
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
    #[command(subcommand)]
    cmd: commands::Commands,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    color_eyre::install()?;
    let opts: Opts = Opts::parse();
    let ctx = context::Context::new_from_options(&opts)?;
    opts.cmd.run(&ctx).await?;
    Ok(())
}
