use url::Url;

mod commands;
mod config;
mod context;
mod print;

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
    /// HTTP Git repository URL (eg. https://github.com/napalmpapalam/rlx)
    #[arg(global = true, short, long)]
    pub repository: Option<Url>,
    /// Path to the workspace directory with the packages directories if it's mono-repo (eg. "./packages") (default: current directory)
    #[arg(global = true, short, long)]
    pub workspace_path: Option<String>,
    /// Enable debug mode
    #[arg(global = true, short, long)]
    pub debug: Option<bool>,
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
