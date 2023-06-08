use crate::config::{AutoPrune, Config};
use crate::continue_loop;
use crate::sources::{Backend, Downloader, ExporterSource};
use clap::{Parser, ValueEnum};
use lib::anyhow::{anyhow, Context, Result};
use lib::cli::Flags;
use lib::simplelog::{debug, info, trace};
use std::fmt::Debug;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = env!["CARGO_PKG_NAME"], version, author, about)]
pub struct Cli {
    #[command(flatten)]
    pub flags: Flags,

    #[command(flatten)]
    pub auto_prune: AutoPrune,
}

/// The main entry point for the application.
/// # Arguments
/// * `directory` - The directory which contains or will contain the backed up data.
pub async fn main(destination: PathBuf, cli: Cli, is_interactive: bool) -> Result<()> {
    if destination.metadata().unwrap().permissions().readonly() {
        Err(anyhow!("Destination is readonly"))?
    }

    if !is_interactive {
        todo!("Non-interactive mode not yet implemented")
    }

    let exporters = prepare(destination.clone(), &is_interactive)?;
    info!("Selected sources: {:?}", &exporters);

    for exporter in &exporters {
        exporter.prune(&destination, &cli.auto_prune)?;
        exporter.download(&destination, &cli.auto_prune).await?;
    }

    // Search the drive for existing backup data
    // Store a database/state file of the currently backed up info
    // Prune the existing files to keep storage free
    // Select files from S3 to download to drive
    // Creates exports for bitwarden, 1password for drive

    save_config(destination, exporters, &is_interactive)?;

    Ok(())
}

// Returns the vec of exporters and a bool which indicates if the config was loaded from disk
fn prepare(directory: PathBuf, interactive: &bool) -> Result<Vec<Box<dyn Backend>>> {
    let config_path = directory.join("settings.json");

    if config_path.exists() {
        debug!("Existing Settings found at {}", &config_path.display());
        let use_existing = inquire::Confirm::new("Do you want to load the existing settings file?")
            .with_default(true)
            .prompt()?;

        if use_existing {
            let config = std::fs::read_to_string(config_path)?;
            let mut config = serde_json::from_str::<Config>(&config).context("Parsing settings.json")?;
            let exporters = config
                .exporters
                .into_iter()
                .filter_map(|mut e| e.create().ok());

            return Ok(exporters.collect());
        }
    }

    if !interactive {
        return Err(anyhow!("No settings found, and not in interactive mode!"));
    }

    let mut exporters = Vec::new();
    loop {
        if continue_loop(&exporters, "Export Source") == false {
            break;
        }

        let source_type = inquire::Select::new(
            "Select your backup source",
            ExporterSource::value_variants().to_vec(),
        )
        .prompt()
        .with_context(|| "Selecting backup source type");

        match source_type {
            Ok(t) => {
                let vec = t.create(&interactive)?;
                exporters.extend(vec);
            }
            Err(_) => {
                trace!("Finished selecting sources");
                break;
            }
        }
    }

    Ok(exporters)
}

fn save_config(root: PathBuf, exporters: Vec<Box<dyn Backend>>, interactive: &bool) -> Result<()> {
    if !interactive {
        return Ok(());
    }

    let current_config = Config {
        rule: Default::default(), // TODO :: Cli / env override
        exporters
    };

    let existing = root.join("settings.json");
    if existing.exists() {
        let existing = std::fs::read_to_string(&existing).context("Reading existing settings")?;
        let existing = serde_json::from_str::<Config>(&existing).context("Parsing existing settings")?;

        if existing == current_config {
            trace!("Settings are the same, not saving");
            return Ok(());
        }
    }

    let save = inquire::Confirm::new("Do you want to save these settings?")
        .with_default(true)
        .prompt()
        .with_context(|| "Prompt for if we should saving settings.")?;

    if !save {
        trace!("Not saving settings");
        return Ok(());
    }

    let destination = root.join("settings.json");
    if destination.exists() {
        let overwrite = inquire::Confirm::new("Do you want to overwrite the existing settings?")
            .with_default(false)
            .prompt()
            .with_context(|| "Prompt for if we should overwrite settings.")?;

        if !overwrite {
            trace!("Not overwriting settings");
            return Ok(());
        }
    }

    let config = Config {
        rule: Default::default(), // TODO :: Cli / env override
        exporters
    };
    let serialised = serde_json::to_string(&config)?;
    info!("Saving settings to {}", &destination.display());
    std::fs::write(&root.join("settings.json"), serialised).context("Saving settings.json")
}
