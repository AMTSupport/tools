use std::error::Error;
use backup::application;
use inquire::PathSelectionMode;
use lib::anyhow::{Result};
use lib::clap::Parser;

use std::path::PathBuf;
use lib::simplelog::trace;

#[cfg(windows)]
const INTERACTIVE_START: &str = r"C:\";
#[cfg(unix)]
const INTERACTIVE_START: &str = "/";

#[tokio::main]
async fn main() -> Result<()> {
    let cli = application::Cli::parse();
    lib::log::init("backup-interactive", &cli.flags)?;
    // let _ = required_elevated_privileges().is_some_and(|code| code.exit());

    // TODO :: Save this in a config on the machine

    let destination = std::env::var("BACKUP_DIR")
        .map(PathBuf::from)
        .or_else(|_| interactive_select_drive())?;

    trace!("Selected destination: {}", &destination.display());

    application::main(destination, cli, true).await?;

    // TODO :: Verify writable
    // TODO :: Verify enough space
    // TODO :: Verify dir is either empty, or has existing backup data

    Ok(())
}

fn interactive_select_drive() -> core::result::Result<PathBuf, impl Error> {
    let drive = inquire::PathSelect::new("Select your backup destination", Some(INTERACTIVE_START))
        .with_selection_mode(PathSelectionMode::Directory)
        .with_select_multiple(false)
        .prompt();

    match drive {
        Ok(drive) => {
            let entry = drive.first().unwrap();
            Ok(entry.path.clone())
        },
        Err(err) => Err(err)
    }
}


#[cfg(windows)]
static DRIVES: Lazy<Vec<PathBuf>> = Lazy::new(|| {
    let mut drives = Vec::with_capacity(26);
    for x in 0..26 {
        let drive = format!("{}:", (x + 64) as u8 as char);
        let drive = PathBuf::from(drive);
        if drive.exists() {
            drives.push(drive);
        }
    }

    return drives;
});
