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

use crate::cleaners::location::Location;
use crate::config::runtime::Runtime;
use crate::rule::Rules;
use macros::{CommonFields, Delegation, EnumVariants};
use rayon::prelude::*;
use std::fmt::Debug;
use std::io;
use std::path::PathBuf;
use clap::ValueEnum;
use thiserror::Error;
use tracing::{trace, warn};

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, EnumVariants, Delegation, ValueEnum)]
#[delegate(trait = CleanerInternal)]
pub enum Cleaner {
    #[delegate(path = crate::cleaners::impls::log::LogCleaner)]
    Logs,
    #[delegate(path = crate::cleaners::impls::browser::BrowserCleaner)]
    Browsers,
    #[delegate(path = crate::cleaners::impls::shader::ShaderCleaner)]
    Shaders,
    #[delegate(path = crate::cleaners::impls::thumbnail::ThumbnailCleaner)]
    Thumbnails,
    #[delegate(path = crate::cleaners::impls::temp::TempCleaner)]
    Temp,
    #[delegate(path = crate::cleaners::impls::downloads::DownloadsCleaner)]
    Downloads,
    #[delegate(path = crate::cleaners::impls::trash::TrashCleaner)]
    Trash,
    // Environment,
    // Cache,
    // Trash,
    // RecycleBin,
    // CrashDumps,
    // OldWindows,
}

pub trait CleanerInternal: Send + Sync + Debug + 'static {
    fn new() -> Self
    where
        Self: Sized;

    fn rules(&self) -> Rules;

    fn locations(&self) -> Vec<Location>;

    /// Returns if the cleaner is able to be run on the current platform.
    fn supported(&self) -> bool {
        true
    }

    fn clean(&self, runtime: &'static Runtime) -> CleanupResult;
}

#[derive(Debug, CommonFields)]
pub enum CleanupResult {
    Cleaned(Cleaner, Vec<CleanedFile>),
    Partial(Cleaner, Vec<CleanedFile>, Vec<MissedFile>),
    Skipped(Cleaner, String),
    Failed(Cleaner, anyhow::Error),
}

impl CleanupResult {
    pub fn extend_missed(self, new_missed: Vec<MissedFile>) -> Self {
        if new_missed.is_empty() {
            return self;
        }

        match self {
            Self::Cleaned(source, cleaned) => Self::Partial(source, cleaned, new_missed),
            Self::Partial(source, cleaned, mut missed) => {
                missed.extend(new_missed);
                Self::Partial(source, cleaned, missed)
            }
            _ => self,
        }
    }

    pub fn extend_cleaned(self, passed: Vec<CleanedFile>) -> Self {
        if passed.is_empty() {
            return self;
        }

        match self {
            Self::Cleaned(source, mut cleaned) => {
                cleaned.extend(passed);
                Self::Cleaned(source, cleaned)
            }
            Self::Partial(source, mut cleaned, missed) => {
                cleaned.extend(passed);
                Self::Partial(source, cleaned, missed)
            }
            _ => self,
        }
    }
}

#[derive(Debug)]
pub struct CleanedFile {
    pub path: PathBuf,
    pub size: u64,
}

#[derive(Error, Debug, CommonFields)]
pub enum MissedFile {
    #[error("The file {0} didn't pass all rule")]
    Rule(PathBuf, u64),
    #[error("The file {0} is currently in use by another process")]
    InUse(PathBuf, u64),
    #[error("Unable to remove file {0} due to missing permissions")]
    Permission(PathBuf, u64),
    #[error("Unable to remove file {0} due to an unknown error")]
    Other(PathBuf, u64, #[source] io::Error),
}

pub(super) fn collect_locations(iter: Vec<Location>, rules: Rules) -> (Vec<PathBuf>, Vec<MissedFile>) {
    let tuple = iter
        .par_iter()
        .inspect(|l| trace!("Collecting paths for {:?}", l))
        .map(Location::get_recursed)
        .inspect(|p| trace!("Paths collected: {:?}", p))
        .flatten()
        .partition::<Vec<PathBuf>, Vec<PathBuf>, _>(|path| rules.iter().all(|rule| rule.test(path)));

    (
        tuple.0,
        tuple.1.into_iter().map(|f| MissedFile::Rule(f.clone(), f.metadata().unwrap().len())).collect(),
    )
}

pub(super) fn clean_files(source: Cleaner, files: Vec<PathBuf>, runtime: &'static Runtime) -> CleanupResult {
    if files.is_empty() {
        return CleanupResult::Skipped(source, "No files to clean".into());
    }

    let mut cleaned = vec![];
    let mut missed = vec![];
    for file in files {
        let metadata = match file.metadata() {
            Ok(m) => m,
            Err(e) => {
                warn!("Failed to get metadata for file: {e}");
                missed.push(MissedFile::Other(file, 0, e));
                continue;
            }
        };

        if runtime.cli.flags.dry_run {
            cleaned.push(CleanedFile {
                path: file,
                size: metadata.len(),
            });
            continue;
        }

        match std::fs::remove_file(&file) {
            Ok(_) => cleaned.push(CleanedFile {
                path: file,
                size: metadata.len(),
            }),
            Err(e) => {
                warn!("Failed to remove file: {e}");
                missed.push(MissedFile::Other(file, metadata.len(), e));
            }
        }
    }

    match missed.is_empty() {
        true => CleanupResult::Cleaned(source, cleaned),
        false => CleanupResult::Partial(source, cleaned, missed),
    }
}
