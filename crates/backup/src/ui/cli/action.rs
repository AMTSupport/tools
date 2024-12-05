/*
 * Copyright (C) 2024. James Draycott me@racci.dev
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, version 3.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.
 * See the GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program. If not, see https://www.gnu.org/licenses/.
 */

use crate::config::backend::Backend;
use crate::config::config::{Config, Error};
use crate::config::rules::Rules;
use crate::config::runtime::Runtime;
use crate::sources::exporter::ExporterSource;
use crate::ui::cli::ui::BackupCli;
use amt_lib::pathed::ensure_directory_exists;
use amt_lib::ui::cli::continue_loop;
use amt_lib::ui::cli::progress;
use amt_lib::ui::cli::ui_inquire::STYLE;
use anyhow::{anyhow, Context, Result};
use clap::Parser;
use inquire::{PathFilter, PathSelect, PathSelectionMode};
use macros::CommonFields;
use obj_builder::buildable::Buildable;
use std::path::PathBuf;
use std::{env, fs};
use tracing::{error, instrument, trace};

#[derive(Debug, Parser, CommonFields)]
pub enum Action {
    /// Create a new backup configuration interactively
    Init {
        /// The path to the backup location root directory.
        #[clap(short = 'D', long, help = "The path to the backup location root directory.")]
        destination: Option<PathBuf>,
    },

    /// Run the backup process with the existing configuration
    Run {
        /// The path to the backup location root directory.
        #[clap(short = 'D', long, help = "The path to the backup location root directory.")]
        destination: Option<PathBuf>,
    },

    /// Modify the configuration interactively
    Modify {
        /// The path to the backup location root directory.
        #[clap(short = 'D', long, help = "The path to the backup location root directory.")]
        destination: Option<PathBuf>,
    },
}

impl Action {
    #[instrument(level = "TRACE")]
    pub async fn initialise(&self) -> Result<Runtime> {
        match self {
            Action::Init { destination } => {
                let config_path = find_backup_config(destination)?;
                if config_path.exists() {
                    return Err(anyhow!(
                        "A configuration already exists at {}, please run modify instead.",
                        config_path.display()
                    ));
                }

                // Safety: We know the parent exists because it's a file.
                Ok(Runtime::wrapping(config_path.parent().unwrap().to_path_buf()))
            }
            Action::Modify { destination } | Action::Run { destination } => {
                let config_path = find_backup_config(destination)?;
                let config = Config::load(&config_path).await?;
                Ok(Runtime {
                    config,
                    // Safety: We know the parent exists because it's a file.
                    directory: config_path.parent().unwrap().to_path_buf(),
                })
            }
        }
    }

    /// Runs the action.
    pub async fn run(&self, cli: &mut BackupCli) -> Result<()> {
        use indicatif::MultiProgress;
        use std::sync::Arc;

        match self {
            Action::Init { .. } => {
                let exporters = new_exporters(cli.runtime.as_ref().unwrap()).await?;
                let rules = <Rules as Buildable>::from(cli).await?;
                let config = &mut cli.runtime.as_mut().unwrap().config;

                config.exporters = exporters;
                config.rules = rules;
                config.mutated = true;

                Ok(())
            }
            Action::Modify { .. } => {
                use inquire::Confirm;
                // TODO :: Use builder stuff for this

                if Confirm::new("Do you want to modify the rules?").with_default(true).prompt()? {
                    let rules = <Rules as Buildable>::from(cli).await?;
                    let config = &mut cli.runtime.as_mut().unwrap().config;

                    config.rules = rules;
                    config.mutated = true;
                }

                // TODO :: Allow removal of existing exporters
                if Confirm::new("Do you want to modify the exporters?")
                    .with_default(true)
                    .prompt()?
                {
                    let exporters = new_exporters(cli.runtime.as_ref().unwrap()).await?;
                    if !exporters.is_empty() {
                        let config = &mut cli.runtime.as_mut().unwrap().config;

                        config.exporters.extend(exporters);
                        config.mutated = true;
                    }
                }

                Ok(())
            }
            Action::Run { .. } => {
                let config = &cli.runtime.as_ref().unwrap().config;
                let multi_bar = Arc::new(MultiProgress::new());
                let total_progress = Arc::new(multi_bar.add(progress::bar(config.exporters.len() as u64)));

                // TODO :: Clean this the fuck up
                let mut handles = vec![];
                for exporter in config.exporters.clone() {
                    let passed_progress = multi_bar.add(progress::spinner());
                    passed_progress.set_message(format!("Running exporter: {exporter}"));

                    let total_progress = total_progress.clone();
                    let multi_bar = multi_bar.clone();
                    let runtime = cli.runtime.as_ref().unwrap();

                    trace!("Running exporter: {}", exporter);
                    let result = exporter.run(runtime, &passed_progress, &multi_bar).await;
                    total_progress.inc(1);
                    passed_progress.finish_and_clear();

                    handles.push(result);
                }

                total_progress.finish_and_clear();
                // let results = futures::future::join_all(handles).await;
                for result in handles {
                    if let Err(e) = result {
                        error!("Error while running exporter: {:?}", &e);
                    }

                    trace!("Finished running exporter successfully");
                }

                Ok(())
            }
        }
    }
}

#[instrument(level = "TRACE")]
fn find_backup_config(destination: &Option<PathBuf>) -> Result<PathBuf> {
    let by_env_or_cwd = Config::find(destination);
    match by_env_or_cwd {
        Ok(path) => Ok(fs::canonicalize(path)?),
        Err(err) => match err {
            Error::Find => select_location(destination).map(|p| p.join(Config::FILENAME)),
            _ => Err(err.into()),
        },
    }
}

#[instrument(level = "TRACE")]
fn select_location(destination: &Option<PathBuf>) -> Result<PathBuf> {
    if let Some(path) = destination {
        ensure_directory_exists(path)?;
        return Ok(path.to_path_buf());
    }

    if let Ok(path) = env::var("BACKUP_DIR").map(PathBuf::from) {
        ensure_directory_exists(&path)?;
        return Ok(path);
    }

    PathSelect::<&str>::new("Select your backup destination")
        .with_render_config(*STYLE)
        .with_selection_mode(PathSelectionMode::Directory(PathFilter::All))
        .with_select_multiple(false)
        .prompt()
        .map_err(|err| err.into())
        .and_then(|vec| vec.first().cloned().ok_or(anyhow!("No path selected")))
        .map_err(|err| anyhow!("Error while selecting path: {}", err))
        .map(|p| p.path)
}

pub(crate) async fn new_exporters(runtime: &Runtime) -> Result<Vec<Backend>> {
    let mut exporters = Vec::new();
    while continue_loop(&exporters, "Export Source") {
        let source_type = inquire::Select::new("Select your backup source", ExporterSource::get_variants())
            .with_render_config(*STYLE)
            .prompt()
            .with_context(|| "Selecting backup source type");

        match source_type {
            Ok(t) => {
                let vec = t.create(runtime).await?;
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

// #[instrument(level = "TRACE", ret, err)]
// fn new_rules() -> Result<Rules> {
//
//
//     let autoprune = if inquire::Confirm::new("Do you want to enable auto-pruning?").with_default(true).prompt()? {
//         let mut autoprune = AutoPrune { ..Default::default() };
//
//         if let Ok(days) = inquire::Text::new("How long do you want to retain backups for?")
//             .with_default(&autoprune.days.to_string())
//             .with_validator(|v: &_| match usize::from_str(v).is_ok() {
//                 true => Ok(Validation::Valid),
//                 false => Ok(Validation::Invalid("Please enter a valid number of days".into())),
//             })
//             .prompt()
//         {
//             autoprune.days = usize::from_str(&days)?;
//         }
//
//         if let Ok(minimum) =
//             inquire::Text::new("How many backups do you want to retain at a minimum, ignoring the age of the backup?")
//                 .with_default(&autoprune.keep_latest.to_string())
//                 .with_validator(|v: &_| match usize::from_str(v).is_ok() {
//                     true => Ok(Validation::Valid),
//                     false => Ok(Validation::Invalid("Please enter a valid number of backups".into())),
//                 })
//                 .prompt()
//         {
//             autoprune.keep_latest = usize::from_str(&minimum)?;
//         }
//
//         Some(autoprune)
//     } else {
//         None
//     };
//
//     Ok(Rules { auto_prune: autoprune })
// }
