use clap::Parser;
use lib::anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::time::SystemTime;

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
    pub fn should_prune(&self, file: &Path, remaining_files: usize) -> Result<bool> {
        let mtime = file.metadata()?.modified()?;
        let now = SystemTime::now();
        let age = now.duration_since(mtime)?;
        let days = chrono::Duration::from_std(age)?.num_days();

        Ok(days > self.keep_for as i64 && remaining_files > self.keep_latest)
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
