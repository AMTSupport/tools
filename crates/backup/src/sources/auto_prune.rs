/*
 * Copyright (C) 2023-2024. James Draycott me@racci.dev
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

use crate::config::runtime::Runtime;
use anyhow::Result;
use indicatif::MultiProgress;
use std::path::PathBuf;

// TODO :: Implement logic from cleaner crate to handle this!
pub trait Prune {
    /// The files which should be possible to prune.
    /// The files returned by this method will be parsed,
    /// Against the `AutoPrune` struct to determine if they should be removed.
    fn files(&self, config: &Runtime) -> Result<Vec<PathBuf>>;

    /// The main prune function.
    /// This function has a common implementation for all sources,
    /// But can be overridden if needed.
    /// # Arguments
    /// * `rules` - The `AutoPrune` struct which contains the rules for pruning.
    /// # Returns
    /// A `Result` with the `Vec<PathBuf>` of the files which were removed.
    fn prune(&self, _config: &Runtime, _progress_bar: &MultiProgress) -> Result<Vec<PathBuf>> {
        // let files = self.files(config).sort_by(|a, b| {
        //     fn chrono(path: &PathBuf) -> Result<DateTime<FixedOffset>> {
        //         let meta = path.metadata().context("Get meta for comparing times")?;
        //         let mtime = meta.modified().context("Get modified time for comparing times")?;
        //
        //         let now = SystemTime::now();
        //         let age = now.duration_since(mtime).context("Get duration since modified time")?;
        //         match Duration::from_std(age) {
        //             Ok(d) => Ok(d)
        //             Err(_) => {}
        //         }
        //         context("Convert std to chrono")
        //     }
        //
        //     let a = a.metadata();
        //     let b = b.metadata();
        //     a.metadata()
        // });
        // let files = self.files(config)?;
        // let mut files = files.iter();
        // let mut removed_files = vec![];
        //
        // // TODO :: Add dry run option.
        // while let Some(file) = files.next() {
        //     if !(config.config.rules.auto_prune.should_prune(file, files.len())?) {
        //         trace!("Pruning rules prevented pruning for: {}", file.display());
        //         continue;
        //     }
        //
        //     trace!("Pruning file: {}", file.display());
        //     if !config.cli.flags.dry_run {
        //         std::fs::remove_file(file)?;
        //     }
        //     removed_files.push(file.clone());
        // }
        //
        // Ok(removed_files)

        Ok(vec![])
    }
}
