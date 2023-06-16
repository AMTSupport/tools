use crate::config::backend::Backend;
use crate::config::rules::AutoPrune;
use crate::config::runtime::RuntimeConfig;
use clap::Parser;
use futures::executor::block_on;
use futures_util::StreamExt;
use indicatif::{MultiProgress, ProgressBar, ProgressDrawTarget};
use lib::anyhow::{anyhow, Result};
use lib::cli::Flags;
use lib::progress;
use lib::progress::{spinner, style_bar, style_spinner};
use lib::simplelog::{debug, error, info, trace};
use std::fmt::Debug;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::task::{block_in_place, spawn_blocking, JoinHandle};
use tokio::{join, spawn};

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
/// TODO Report at the end of the run what has being exported, and what was pruned
pub async fn main(destination: PathBuf, cli: Cli, is_interactive: bool) -> Result<()> {
    if destination.metadata().unwrap().permissions().readonly() {
        Err(anyhow!("Destination is readonly"))?
    }

    if !is_interactive {
        todo!("Non-interactive mode not yet implemented")
    }

    let config = Arc::new(RuntimeConfig::new(cli, destination).await?);
    debug!("Config: {:?}", config);

    let multi_bar = Arc::new(MultiProgress::new());
    let total_progress =
        Arc::new(multi_bar.add(progress::bar(config.config.exporters.len() as u64)));

    let mut handles = vec![];
    for exporter in config.config.exporters.clone() {
        let passed_progress = multi_bar.add(spinner());
        passed_progress.set_message(format!("Running exporter: {}", exporter));

        let total_progress = total_progress.clone();
        let config = config.clone();
        let multi_bar = multi_bar.clone();

        let handle = spawn(async move {
            let result = exporter.run(&config, &passed_progress, &multi_bar).await;
            total_progress.inc(1);
            passed_progress.finish_and_clear();
            result
        });

        handles.push(handle);
    }

    let results = futures::future::join_all(handles).await;
    for result in results {
        if let Err(e) = result? {
            error!("Error while running exporter: {:?}", e);
        }
    }

    total_progress.finish_and_clear();

    // for result in results {
    //     if !job.is_finished() {
    //         error!("Exporter failed to finish");
    //         continue;
    //     }
    //
    //     if job.await.is_err() {
    //         error!("Error while running exporter: {:?}", job.await.err());
    //         continue;
    //     }
    // }

    // TODO :: Store errors and continue; report them at the end
    // TODO :: Maybe pass progress bar to exporters for better UX
    // let mut errors = vec![];
    // for mut e in config.config.exporters.clone() {
    //     info!("Running exporter: {}", e);
    //     match e.run(&config).await {
    //         Ok(_) => trace!("Exporter finished successfully"),
    //         Err(err) => {
    //             trace!("Exporter failed");
    //             errors.push((e, err));
    //         }
    //     }
    // }

    // if let Err(e) = config.save() {
    //     error!("Failed to save config: {}", e);
    // }
    //
    // if !errors.is_empty() {
    //     Err(anyhow!("Some exporters failed: {:?}", errors))?
    // }

    Ok(())
}
