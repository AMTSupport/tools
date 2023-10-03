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
use async_trait::async_trait;
use clap::ValueEnum;
use macros::{CommonFields, Delegation, EnumNames, EnumVariants};
use rayon::prelude::*;
use std::fmt::Debug;
use std::io;
use std::path::PathBuf;
use thiserror::Error;
use tokio_stream::StreamExt;
use tracing::{instrument, trace, warn};

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Delegation, EnumVariants, ValueEnum, EnumNames)]
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
}

#[async_trait]
pub trait CleanerInternal: Debug + Send + Sync + 'static {
    fn new() -> Self
    where
        Self: Sized + Default,
    {
        Self::default()
    }

    fn rules(&self) -> Rules;

    fn locations(&self) -> Vec<Location>;

    /// Returns if the cleaner is able to be run on the current platform.
    fn supported(&self) -> bool {
        true
    }

    async fn clean(&self, runtime: &'static Runtime) -> CleanupResult;
}

#[derive(Debug, CommonFields)]
pub enum CleanupResult {
    Cleaned {
        cleaner: Cleaner,
        cleaned: Vec<CleanedFile>,
    },
    Partial {
        cleaner: Cleaner,
        cleaned: Vec<CleanedFile>,
        missed: Vec<MissedFile>,
    },
    Skipped {
        cleaner: Cleaner,
        reason: SkipReason,
    },
    Failed {
        cleaner: Cleaner,
        source: anyhow::Error,
    },
}

impl CleanupResult {
    pub fn size(&self) -> u64 {
        match self {
            Self::Cleaned { cleaned, .. } => cleaned.iter().map(|f| f.size).sum(),
            Self::Partial { cleaned, .. } => cleaned.iter().map(|f| f.size).sum(),
            _ => 0,
        }
    }

    pub fn cleaned_count(&self) -> u64 {
        match self {
            Self::Cleaned { cleaned, .. } => cleaned.len() as u64,
            Self::Partial { cleaned, .. } => cleaned.len() as u64,
            _ => 0,
        }
    }

    pub fn missed_count(&self) -> u64 {
        match self {
            Self::Partial { missed, .. } => missed.len() as u64,
            _ => 0,
        }
    }

    pub fn missed_size(&self) -> u64 {
        match self {
            Self::Partial { missed, .. } => missed.iter().map(|f| f.field_1()).sum(),
            _ => 0,
        }
    }
}

#[derive(Debug, PartialEq, Error)]
pub enum SkipReason {
    #[error("the cleaner is not supported on this platform.")]
    Unsupported,

    #[error("there are no files that can be cleaned.")]
    NoFiles,
}

impl CleanupResult {
    pub fn extend_missed(self, new_missed: Vec<MissedFile>) -> Self {
        if new_missed.is_empty() {
            return self;
        }

        match self {
            Self::Cleaned { cleaner, cleaned } => Self::Partial {
                cleaner,
                cleaned,
                missed: new_missed,
            },
            Self::Partial {
                cleaner,
                cleaned,
                mut missed,
            } => {
                missed.extend(new_missed);
                Self::Partial {
                    cleaner,
                    cleaned,
                    missed,
                }
            }
            Self::Skipped {
                cleaner,
                reason: SkipReason::NoFiles,
            } => Self::Partial {
                cleaner,
                cleaned: vec![],
                missed: new_missed,
            },
            Self::Skipped { .. } | Self::Failed { .. } => self,
        }
    }

    pub fn extend_cleaned(self, passed: Vec<CleanedFile>) -> Self {
        if passed.is_empty() {
            return self;
        }

        match self {
            Self::Cleaned { cleaner, mut cleaned } => {
                cleaned.extend(passed);
                Self::Cleaned { cleaner, cleaned }
            }
            Self::Partial {
                cleaner,
                mut cleaned,
                missed,
            } => {
                cleaned.extend(passed);
                Self::Partial {
                    cleaner,
                    cleaned,
                    missed,
                }
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

#[instrument(level = "TRACE", skip(iter))]
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

#[instrument(level = "TRACE", skip(files, runtime))]
pub(super) async fn clean_files(cleaner: Cleaner, files: Vec<PathBuf>, runtime: &'static Runtime) -> CleanupResult {
    if files.is_empty() {
        return CleanupResult::Skipped {
            cleaner,
            reason: SkipReason::NoFiles,
        };
    }

    let mut cleaned = vec![];
    let mut missed = vec![];
    let mut result_stream = tokio_stream::iter(files).map(|file| {
        let metadata = match file.metadata() {
            Ok(m) => m,
            Err(e) => {
                warn!("Failed to get metadata for file: {e}");
                return Err(MissedFile::Other(file, 0, e));
            }
        };

        if runtime.cli.flags.dry_run {
            return Ok(CleanedFile {
                path: file,
                size: metadata.len(),
            });
        }

        match std::fs::remove_file(&file) {
            Ok(_) => Ok(CleanedFile {
                path: file,
                size: metadata.len(),
            }),
            Err(e) => match e.kind() {
                io::ErrorKind::ExecutableFileBusy => Err(MissedFile::InUse(file, metadata.len())),
                io::ErrorKind::PermissionDenied => Err(MissedFile::Permission(file, metadata.len())),
                _ => {
                    warn!("Failed to remove file due to unknown error: {e}");
                    Err(MissedFile::Other(file, metadata.len(), e))
                }
            },
        }
    });

    while let Some(result) = result_stream.next().await {
        match result {
            Ok(cleaned_file) => cleaned.push(cleaned_file),
            Err(missed_file) => missed.push(missed_file),
        }
    }

    match missed.is_empty() {
        true => CleanupResult::Cleaned { cleaner, cleaned },
        false => CleanupResult::Partial {
            cleaner,
            cleaned,
            missed,
        },
    }
}

pub(super) async fn basic_files<C: CleanerInternal>(
    relational: Cleaner,
    cleaner: &C,
    runtime: &'static Runtime,
) -> CleanupResult {
    let (passed, failed) = collect_locations(cleaner.locations(), cleaner.rules());
    let passed_result = clean_files(relational, passed, runtime).await;

    passed_result.extend_missed(failed)
}
