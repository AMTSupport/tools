use clap::Parser;
use cleaner::application::application;
use lib::{cli::Flags, log};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Flags::parse();
    log::init(env!["CARGO_PKG_NAME"], &cli)?;

    application(cli).await?;

    Ok(())
}
