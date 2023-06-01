#![feature(async_closure)]

use clap::Parser;
use cleaner::application::{self, application};
use lib::log as Logger;
use simplelog::error;

const EXIT_CODE_INVALID_PERMISSIONS: i32 = 4;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = application::Cli::parse();
    Logger::init(env!["CARGO_PKG_NAME"], &cli.flags)?;

    #[cfg(windows)]
    if is_elevated::is_elevated() == false {
        error!("This application must be run as an administrator");
        std::process::exit(EXIT_CODE_INVALID_PERMISSIONS);
    }

    #[cfg(unix)]
    if nix::unistd::geteuid().is_root() == false {
        error!("This application must be run as root");
        std::process::exit(EXIT_CODE_INVALID_PERMISSIONS);
    }

    application(cli).await?;

    Ok(())
}
