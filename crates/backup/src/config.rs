use crate::application::Cli;
use crate::continue_loop;
use crate::sources::auto_prune::Prune;
use crate::sources::bitwarden::BitWardenCore;
use crate::sources::exporter::{Exporter, ExporterSource};
use crate::sources::s3::S3Core;
use clap::Parser;
use clap::ValueEnum;
use inquire::validator::Validation;
use lib::anyhow::{Context, Result};
use lib::simplelog::{debug, info, trace};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::str::FromStr;
use std::time::SystemTime;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Backend {
    S3(S3Core),
    BitWarden(BitWardenCore),
}

impl Backend {
    pub async fn run(&mut self, config: &RuntimeConfig) -> Result<()> {
        match self {
            Backend::S3(ref mut core) => {
                core.prune(&config)?;
                core.export(&config).await?;
            }
            Backend::BitWarden(ref mut core) => {
                core.prune(&config)?;
                core.export(&config).await?;
            }
        }

        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct RuntimeConfig {
    // If this config requires saving.
    pub mutated: bool,
    pub directory: PathBuf,
    pub cli: Cli,
    pub config: Config,
}

impl RuntimeConfig {
    pub(crate) fn new(cli: Cli, directory: PathBuf) -> Result<Self> {
        let config_path = directory.join("settings.json");

        if config_path.exists() {
            debug!("Existing Settings found at {}", &config_path.display());

            if inquire::Confirm::new("Do you want to load the existing settings file?")
                .with_default(true)
                .prompt()
                .is_ok_and(|b| b)
            {
                let mut config = Self {
                    mutated: false,
                    directory,
                    cli,
                    config: std::fs::read(config_path)
                        .context("Reading settings.json")
                        .and_then(|vec| {
                            serde_json::from_slice::<Config>(&vec).context("Parsing settings.json")
                        })?,
                };

                if config.cli.append {
                    let exporters = Self::new_exporters(&config)?;
                    if !exporters.is_empty() {
                        config.config.exporters.extend(exporters);
                        config.mutated = true;
                    }
                }

                return Ok(config);
            }
        }

        let mut config = RuntimeConfig {
            mutated: true,
            directory,
            cli,
            config: Config {
                rules: Self::new_rules()?,
                exporters: vec![],
            },
        };

        config.config.exporters = Self::new_exporters(&config)?;
        Ok(config)
    }

    pub(crate) fn save(self) -> Result<()> {
        if !self.mutated {
            return Ok(());
        }

        if inquire::Confirm::new("Do you want to save these settings?")
            .with_default(true)
            .prompt()
            .is_ok_and(|b| !b)
        {
            trace!("Not saving settings");
            return Ok(());
        }

        let destination = self.directory.join("settings.json");
        if destination.exists() {
            let overwrite =
                inquire::Confirm::new("Do you want to overwrite the existing settings?")
                    .with_default(false)
                    .prompt()
                    .context("Prompt for if we should overwrite settings.")?;

            if !overwrite {
                trace!("Not overwriting settings");
                return Ok(());
            }
        }

        let serialised = serde_json::to_string_pretty(&self.config)?;
        info!("Saving settings to {}", &destination.display());
        std::fs::write(destination, serialised).context("Saving settings.json")
    }

    fn new_rules() -> Result<Rules> {
        trace!("Inquiring about rules");

        let prune = if inquire::Confirm::new("Do you want to enable auto-pruning?")
            .with_default(true)
            .prompt()?
        {
            let mut prune = AutoPrune::default();
            prune.enabled = true;

            if let Ok(days) = inquire::Text::new("How long do you want to retain backups for?")
                .with_default(&prune.keep_for.to_string())
                .with_validator(|v: &_| match usize::from_str(v).is_ok() {
                    true => Ok(Validation::Valid),
                    false => Ok(Validation::Invalid(
                        "Please enter a valid number of days".into(),
                    )),
                })
                .prompt()
            {
                prune.keep_for = usize::from_str(&days)?;
            }

            if let Ok(minimum) = inquire::Text::new(
                "How many backups do you want to retain at a minimum, ignoring the age of the backup?",
            )
                .with_default(&prune.keep_latest.to_string())
                .with_validator(|v: &_| match usize::from_str(v).is_ok() {
                    true => Ok(Validation::Valid),
                    false => Ok(Validation::Invalid(
                        "Please enter a valid number of backups".into(),
                    )),
                })
                .prompt()
            {
                prune.keep_latest = usize::from_str(&minimum)?;
            }

            prune
        } else {
            AutoPrune::default()
        };

        Ok(Rules { auto_prune: prune })
    }

    fn new_exporters(config: &RuntimeConfig) -> Result<Vec<Backend>> {
        let mut exporters = Vec::new();
        while continue_loop(&exporters, "Export Source") {
            let source_type = inquire::Select::new(
                "Select your backup source",
                ExporterSource::value_variants().to_vec(),
            )
            .prompt()
            .with_context(|| "Selecting backup source type");

            match source_type {
                Ok(t) => {
                    let vec = t.create(&config)?;
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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub rules: Rules,
    pub exporters: Vec<Backend>,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct Rules {
    /// The AutoPrune configuration.
    pub auto_prune: AutoPrune,
}

#[derive(Parser, Debug, Clone, Serialize, Deserialize)]
pub struct AutoPrune {
    /// Whether or not the auto prune feature is enabled.
    #[arg(long = "prune", action = clap::ArgAction::SetTrue)]
    pub enabled: bool,

    /// How long backups should be kept for in days.
    #[arg(long = "prune-keep-days", default_value = "28")]
    pub keep_for: usize,

    /// The minimum number of backups to keep ignoring the keep_for duration.
    #[arg(long = "prune-keep-count", default_value = "5")]
    pub keep_latest: usize,
}

impl AutoPrune {
    pub fn should_prune(&self, file: &PathBuf, remaining_files: usize) -> Result<bool> {
        let mtime = file.metadata()?.modified()?;
        let now = SystemTime::now();
        let age = now.duration_since(mtime)?;
        let days = chrono::Duration::from_std(age)?.num_days();

        Ok(days > self.keep_for.clone() as i64 && remaining_files > *&self.keep_latest)
    }
}

impl Default for AutoPrune {
    fn default() -> Self {
        Self {
            enabled: false,
            keep_for: 28,
            keep_latest: 5,
        }
    }
}
