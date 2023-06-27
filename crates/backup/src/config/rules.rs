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

use lib::anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::time::SystemTime;

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct Rules {
    /// The AutoPrune configuration.
    pub auto_prune: AutoPrune,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoPrune {
    /// Whether or not the auto prune feature is enabled.
    pub enabled: bool,

    /// How many days of backups should be kept.
    pub days: usize,

    /// How many per week backups should be kept.
    pub weeks: usize,

    /// How many per month backups should be kept.
    pub months: usize,

    /// The minimum number of backups to keep ignoring the keep_for duration.
    pub keep_latest: usize,
}

impl AutoPrune {
    pub fn should_prune(&self, file: &Path, remaining_files: usize) -> Result<bool> {
        let mtime = file.metadata()?.modified()?;
        let now = SystemTime::now();
        let age = now.duration_since(mtime)?;
        let days = chrono::Duration::from_std(age)?.num_days();

        Ok(days > self.days as i64 && remaining_files > self.keep_latest)
    }
}

impl Default for AutoPrune {
    fn default() -> Self {
        Self {
            enabled: false,
            days: 14,
            weeks: 0,
            months: 0,
            keep_latest: 5,
        }
    }
}
