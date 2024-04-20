use clap::Parser;
use url::Url;

mod commands;
mod config;
mod context;

#[derive(Clone, Parser, Debug)]
#[clap(about)]
pub(crate) struct Opts {
    /// HTTP Git repository URL (eg. https://github.com/napalmpapalam/rlx)
    #[arg(global = true, short, long)]
    pub(crate) repository: Option<Url>,
    #[command(subcommand)]
    cmd: commands::Commands,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let opts: Opts = Opts::parse();
    let ctx = context::Context::new_from_options(&opts)?;
    opts.cmd.run(&ctx).await?;
    Ok(())
}
