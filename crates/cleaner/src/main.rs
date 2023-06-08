#![feature(async_closure)]

use clap::Parser;
use cleaner::application::{self, application};
use lib::anyhow::Result;
use lib::helper::required_elevated_privileges;
use lib::log as Logger;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = application::Cli::try_parse()?;
    Logger::init(env!["CARGO_PKG_NAME"], &cli.flags)?;
    let _ = required_elevated_privileges().is_some_and(|code| code.exit());

    application(cli).await?;

    Ok(())
}
