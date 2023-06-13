use crate::config::rules::AutoPrune;
use crate::config::runtime::RuntimeConfig;
use clap::Parser;
use lib::anyhow::{anyhow, Result};
use lib::cli::Flags;
use lib::simplelog::{debug, error, info, trace};
use std::fmt::Debug;
use std::path::PathBuf;

#[derive(Parser, Debug, Clone)]
#[command(name = env!["CARGO_PKG_NAME"], version, author, about)]
pub struct Cli {
    #[command(flatten)]
    pub flags: Flags,

    #[command(flatten)]
    pub auto_prune: AutoPrune,

    #[arg(short, long, action = clap::ArgAction::SetTrue)]
    pub append: bool,
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

    let config = RuntimeConfig::new(cli, destination).await?;
    debug!("Config: {:?}", config);

    // TODO :: Store errors and continue; report them at the end
    // TODO :: Maybe pass progress bar to exporters for better UX
    let mut errors = vec![];
    for mut e in config.config.exporters.clone() {
        info!("Running exporter: {}", e);
        match e.run(&config).await {
            Ok(_) => trace!("Exporter finished successfully"),
            Err(err) => {
                trace!("Exporter failed");
                errors.push((e, err));
            }
        }
    }

    if let Err(e) = config.save() {
        error!("Failed to save config: {}", e);
    }

    if !errors.is_empty() {
        Err(anyhow!("Some exporters failed: {:?}", errors))?
    }

    Ok(())
}
