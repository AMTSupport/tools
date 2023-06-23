#![feature(result_option_inspect)]

use backup::application;
use lib::anyhow::{Context, Result};
use lib::clap::Parser;
use tracing::{error, trace};
use std::env;
use std::env::VarError;
use std::path::PathBuf;

#[tokio::main(flavor = "multi_thread")]
pub async fn main() -> Result<()> {
    let cli = application::Cli::try_parse()?;
    lib::log::init("backup-interactive", &cli.flags)?;
    rayon::ThreadPoolBuilder::new().num_threads(22).build_global().unwrap();
    application::main(select_location()?, cli, true).await

    // TODO :: Verify writable
    // TODO :: Verify enough space
    // TODO :: Verify dir is either empty, or has existing backup data
}

// TODO :: maybe drop this and have the binary placed in the directory where it will be used?
fn select_location() -> Result<PathBuf> {
    let working_dir = env::current_dir().context("Failed to get current directory")?;
    if working_dir.join("settings.json").exists() {
        trace!("Running from working dir {}", working_dir.display());
        return Ok(working_dir);
    }

    env::var("BACKUP_DIR")
        .map(PathBuf::from)
        // TODO :: Verify writable
        .and_then(|path| if path.exists() { Ok(path) } else { Err(VarError::NotPresent) })
        .inspect_err(|_err| {
            error!("The path specified in BACKUP_DIR does not exist.");
            error!("Please fix this, or unset the BACKUP_DIR environment variable to use the interactive mode.");
        }).context("Failed to get backup directory from BACKUP_DIR environment variable")
    // .or_else(|_| PathSelect::<&str>::new("Select your backup destination", None)
    //     .with_selection_mode(PathSelectionMode::Directory)
    //     .with_select_multiple(false)
    //     .prompt()
    //     .map_err(|err| anyhow!("No path selected or error while selecting path: {}", err))
    //     .map(|vec| vec.first().unwrap().path.clone()))
}
