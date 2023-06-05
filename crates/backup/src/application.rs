use clap::{Parser, ValueEnum};
use lib::anyhow::{anyhow, Result};
use lib::cli::Flags;
use lib::simplelog::{info, trace};
use std::fmt::Debug;
use std::path::PathBuf;
use crate::sources::ExporterSource;

#[derive(Parser, Debug)]
#[command(name = env!["CARGO_PKG_NAME"], version, author, about)]
pub struct Cli {
    #[command(flatten)]
    pub flags: Flags,
}

/// The main entry point for the application.
/// # Arguments
/// * `directory` - The directory which contains or will contain the backed up data.
pub async fn main(destination: PathBuf, is_interactive: bool) -> Result<()> {
    if destination.metadata().unwrap().permissions().readonly() {
        Err(anyhow!("Destination is readonly"))?
    }

    if !is_interactive {
        todo!("Non-interactive mode not yet implemented")
    }

    let mut sources = Vec::new();
    loop {
        if !sources.is_empty() {
            let more = inquire::Confirm::new("Do you want to add another source?")
                .with_default(false)
                .prompt()?;

            if more == false {
                break;
            }
        }

        let source = match inquire::Select::new(
            "Select your backup source",
            ExporterSource::value_variants().to_vec(),
        )
        .prompt()
        {
            Err(_) => {
                trace!("Finished selecting sources");
                break;
            }
            Ok(source) => source,
        };

        sources.extend(source.create(&is_interactive, &destination)?);
    }

    info!("Selected sources: {:#?}", &sources);

    // Search the drive for existing backup data
    // Store a database/state file of the currently backed up info
    // Prune the existing files to keep storage free
    // Select files from S3 to download to drive
    // Creates exports for bitwarden, 1password for drive

    if is_interactive {
        match inquire::Confirm::new("Do you want to save these settings?").with_default(true).prompt() {
            Ok(true) => {
                let serialised = serde_json::to_string(&sources.first().unwrap())?;
            }
            _ => trace!("Not saving settings");

        }
    }

    Ok(())
}
