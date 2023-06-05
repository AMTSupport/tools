#![feature(async_closure)]

use clap::Parser;
use cleaner::application::{self, application};
use lib::anyhow::Result;
use lib::exitcode::is_error;
use lib::helper::{required_elevated_privileges, required_elevated_privileges_or_exit};
use lib::sysexits::ExitCode;
use lib::{log as Logger, sysexits};
use simplelog::error;
use std::process::exit;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = application::Cli::parse();
    Logger::init(env!["CARGO_PKG_NAME"], &cli.flags)?;
    required_elevated_privileges().if_some(|code| exit(code));

    application(cli).await?;

    Ok(())
}
