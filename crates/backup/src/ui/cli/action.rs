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

use crate::config::backend::Backend;
use crate::config::config::Config;
use crate::config::rules::autoprune::AutoPrune;
use crate::config::rules::Rules;
use crate::config::runtime::Runtime;
use crate::sources::exporter::ExporterSource;
use crate::ui::cli::continue_loop;
use anyhow::{anyhow, Context, Result};
use inquire::validator::Validation;
use inquire::{PathFilter, PathSelect, PathSelectionMode};
use lib::pathed::ensure_directory_exists;
use lib::ui::cli::progress;
use std::env;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use tracing::{error, instrument, trace};

#[derive(Debug, Clone, Copy, clap::Subcommand)]
pub enum Action {
    /// Create a new backup configuration interactively
    Init,

    /// Run the backup process with the existing configuration
    Run,

    /// Modify the configuration interactively
    Modify,
}

impl Action {
    #[instrument(level = "TRACE")]
    pub fn prepare(&self, destination: Option<&Path>) -> Result<Runtime> {
        match self {
            Action::Init => {
                let backup_root = find_backup_root(None)?;
                let config_path = backup_root.join(Config::FILENAME);
                if config_path.exists() {
                    return Err(anyhow!(
                        "A configuration already exists at {}, please run modify instead.",
                        config_path.display()
                    ));
                }

                Ok(Runtime::wrapping(backup_root))
            }
            Action::Modify | Action::Run => {
                let backup_root = find_backup_root(destination)?;
                Ok(Runtime::wrapping(backup_root))
            }
        }
    }

    /// Runs the action.
    pub async fn run(&self, runtime: &mut Runtime) -> Result<()> {
        use indicatif::MultiProgress;
        use std::sync::Arc;

        match self {
            Action::Init => {
                runtime.config.rules = new_rules()?;
                runtime.config.exporters = new_exporters(&runtime).await?;
                runtime.config.mutated = true;

                Ok(())
            }
            Action::Modify => {
                use inquire::Confirm;
                // TODO :: Use builder stuff for this

                if Confirm::new("Do you want to modify the rules?").with_default(true).prompt()? {
                    runtime.config.rules = new_rules()?;
                    runtime.config.mutated = true;
                }

                // TODO :: Allow removal of existing exporters
                if Confirm::new("Do you want to modify the exporters?").with_default(true).prompt()? {
                    let exporters = new_exporters(&runtime).await?;
                    if !exporters.is_empty() {
                        runtime.config.exporters.extend(exporters);
                        runtime.config.mutated = true;
                    }
                }

                Ok(())
            }
            Action::Run => {
                let multi_bar = Arc::new(MultiProgress::new());
                let total_progress = Arc::new(multi_bar.add(progress::bar(runtime.config.exporters.len() as u64)));

                // TODO :: Clean this the fuck up
                let mut handles = vec![];
                for exporter in runtime.config.exporters.clone() {
                    let passed_progress = multi_bar.add(progress::spinner());
                    passed_progress.set_message(format!("Running exporter: {exporter}"));

                    let total_progress = total_progress.clone();
                    let multi_bar = multi_bar.clone();
                    let runtime = runtime.clone();

                    trace!("Running exporter: {}", exporter);
                    let result = exporter.run(&runtime, &passed_progress, &multi_bar).await;
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

fn find_backup_root(destination: Option<&Path>) -> Result<PathBuf> {
    match select_location(destination) {
        Ok(path) => Ok(path),
        Err(err) => {
            error!("Error while selecting backup location: {}", err);
            let path = Config::find(destination)?;
            match path.parent() {
                Some(parent) => Ok(parent.to_path_buf()),
                None => Err(anyhow!("Unable to find parent directory of {}", path.display())),
            }
        }
    }
}

#[instrument(level = "TRACE")]
fn select_location(destination: Option<&Path>) -> Result<PathBuf> {
    if let Some(path) = destination {
        ensure_directory_exists(path)?;
        return Ok(path.to_path_buf());
    }

    if let Ok(path) = env::var("BACKUP_DIR").map(PathBuf::from) {
        ensure_directory_exists(&path)?;
        return Ok(path);
    }

    PathSelect::<&str>::new("Select your backup destination")
        .with_selection_mode(PathSelectionMode::Directory(PathFilter::All))
        .with_select_multiple(false)
        .prompt()
        .map_err(|err| anyhow!("No path selected or error while selecting path: {}", err))
        .map(|vec| vec.first().unwrap().path.clone())
}

pub(crate) async fn new_exporters(runtime: &Runtime) -> Result<Vec<Backend>> {
    let mut exporters = Vec::new();
    while continue_loop(&exporters, "Export Source") {
        let source_type = inquire::Select::new("Select your backup source", ExporterSource::get_variants())
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

fn new_rules() -> Result<Rules> {
    trace!("Inquiring about rules");

    let autoprune = if inquire::Confirm::new("Do you want to enable auto-pruning?").with_default(true).prompt()? {
        let mut autoprune = AutoPrune { ..Default::default() };

        if let Ok(days) = inquire::Text::new("How long do you want to retain backups for?")
            .with_default(&autoprune.days.to_string())
            .with_validator(|v: &_| match usize::from_str(v).is_ok() {
                true => Ok(Validation::Valid),
                false => Ok(Validation::Invalid("Please enter a valid number of days".into())),
            })
            .prompt()
        {
            autoprune.days = usize::from_str(&days)?;
        }

        if let Ok(minimum) =
            inquire::Text::new("How many backups do you want to retain at a minimum, ignoring the age of the backup?")
                .with_default(&autoprune.keep_latest.to_string())
                .with_validator(|v: &_| match usize::from_str(v).is_ok() {
                    true => Ok(Validation::Valid),
                    false => Ok(Validation::Invalid("Please enter a valid number of backups".into())),
                })
                .prompt()
        {
            autoprune.keep_latest = usize::from_str(&minimum)?;
        }

        Some(autoprune)
    } else {
        None
    };

    Ok(Rules { auto_prune: autoprune })
}
