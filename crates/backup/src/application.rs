/*
 * Copyright (C) 2023 James Draycott <me@racci.dev>
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, version 3.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */

use crate::config::runtime::RuntimeConfig;
use anyhow::{anyhow, Result};
use clap::{Parser, Subcommand};
use indicatif::MultiProgress;
use lib::cli::Flags;
use lib::progress;
use lib::progress::spinner;
use std::fmt::Debug;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::spawn;
use tracing::error;
use tracing::{span, Instrument};

#[derive(Parser, Debug, Clone)]
#[command(name = env!["CARGO_PKG_NAME"], version, author, about)]
pub struct Cli {
    #[command(flatten)]
    pub flags: Flags,

    #[command(subcommand)]
    pub action: Action,
}

#[derive(Debug, Clone, Subcommand)]
pub enum Action {
    /// Create a new backup configuration interactively
    Init,

    /// Run the backup process with the existing configuration
    Run,

    /// Modify the configuration interactively
    Modify,
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

    match cli.action {
        Action::Init => {
            RuntimeConfig::new(&cli, &destination).await?;
        }
        Action::Modify => {
            RuntimeConfig::modify(&cli, &destination).await?;
        }
        Action::Run => {
            let config = RuntimeConfig::get(cli, destination).await?;
            let multi_bar = Arc::new(MultiProgress::new());
            let total_progress = Arc::new(multi_bar.add(progress::bar(config.config.exporters.len() as u64)));

            let mut handles = vec![];
            for exporter in config.config.exporters.clone() {
                let passed_progress = multi_bar.add(spinner());
                passed_progress.set_message(format!("Running exporter: {exporter}"));

                let total_progress = total_progress.clone();
                let config = config.clone();
                let multi_bar = multi_bar.clone();

                let handle = spawn(async move {
                    tracing::trace!("Running exporter: {}", exporter);
                    let result = exporter.run(&config, &passed_progress, &multi_bar).await;
                    total_progress.inc(1);
                    passed_progress.finish_and_clear();
                    result
                })
                .instrument(span!(tracing::Level::TRACE, "exporter"));

                handles.push(handle);
            }

            total_progress.finish_and_clear();
            let results = futures::future::join_all(handles).await;
            for result in results {
                if let Err(e) = result? {
                    error!("Error while running exporter: {:?}", &e);
                }
            }
        }
    }

    Ok(())
}
