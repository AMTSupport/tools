use crate::sources::bitwarden::BitWardenCore;
use crate::sources::s3::S3Core;
use clap::Parser;
use lib::anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::SystemTime;
use crate::sources::auto_prune::Prune;
use crate::sources::exporter::Exporter;

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub enum Backend {
    S3(S3Core),
    BitWarden(BitWardenCore)
}

impl Backend {
    pub async fn run(&mut self, root_directory: &PathBuf, rules: &AutoPrune) -> Result<()> {
        match self {
            Backend::S3(ref mut core) => {
                core.prune(root_directory, rules)?;
                core.export(root_directory, rules).await?;
            },
            Backend::BitWarden(ref mut core) => {
                core.prune(root_directory, rules)?;
                core.export(root_directory, rules).await?;
            }
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct Config {
    #[serde(default)]
    pub rule: Rules,
    pub exporters: Vec<Backend>,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct Rules {
    /// The AutoPrune configuration.
    pub auto_prune: AutoPrune,
}

#[derive(Parser, Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
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
