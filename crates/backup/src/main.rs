#![feature(result_option_inspect)]

use backup::application;
use inquire::{PathSelect, PathSelectionMode};
use lib::anyhow::{anyhow, Context, Result};
use lib::clap::Parser;
use lib::simplelog::{error, trace};
use std::env;
use std::env::VarError;
use std::error::Error;
use std::ops::Deref;
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = application::Cli::parse();
    lib::log::init("backup-interactive", &cli.flags)?;
    // let _ = required_elevated_privileges().is_some_and(|code| code.exit());

    // TODO :: Save this in a config on the machine

    let destination = select_location()?;
    trace!("Selected destination: {}", &destination.display());

    application::main(destination, cli, true).await

    // TODO :: Verify writable
    // TODO :: Verify enough space
    // TODO :: Verify dir is either empty, or has existing backup data
}

// TODO :: maybe drop this and have the binary placed in the directory where it will be used?
fn select_location() -> Result<PathBuf> {
    env::var("BACKUP_DIR")
        .map(PathBuf::from)
        // TODO :: Verify writable
        .and_then(|path| if path.exists() { Ok(path) } else { Err(VarError::NotPresent) })
        .inspect_err(|err| {
            error!("The path specified in BACKUP_DIR does not exist.");
            error!("Please fix this, or unset the BACKUP_DIR environment variable to use the interactive mode.");
        })
        .or_else(|_| PathSelect::<&str>::new("Select your backup destination", None)
            .with_selection_mode(PathSelectionMode::Directory)
            .with_select_multiple(false)
            .prompt()
            .map_err(|err| anyhow!("No path selected or error while selecting path: {}", err))
            .map(|vec| vec.first().unwrap().path.clone()))
}
