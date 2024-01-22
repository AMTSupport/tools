/*
 * Copyright (c) 2023-2024. James Draycott <me@racci.dev>
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
 * You should have received a copy of the GNU General Public License along with this program.
 * If not, see <https://www.gnu.org/licenses/>.
 */

pub(crate) mod age;

use anyhow::Result;
use chrono::Duration;
use std::fmt::Debug;
use std::fs::Metadata;
use std::io;
use std::path::{Path, PathBuf};
use thiserror::Error;
use tracing::{error, instrument, trace};

pub type RuleResult<T> = Result<T, RuleError>;
pub type Rules = Vec<Rule>;

#[derive(Error, Debug)]
pub enum RuleError {
    #[error("getting metadata for {1}: {0}")]
    MetadataError(#[source] io::Error, PathBuf),

    #[error("parsing time {1}: {0}")]
    TimeMetaError(#[source] Box<dyn std::error::Error>, PathBuf),
}

#[derive(Debug, Clone, Copy)]
pub enum Rule {
    Age(Duration, age::Since),
    None,
}

impl Rule {
    /// Returns true if the paths passes the rule
    #[instrument(level = "TRACE")]
    pub fn test(&self, path: &Path) -> bool {
        let passed = match self {
            Rule::Age(..) => age::test(*self, path),
            Rule::None => true,
        };

        trace!("Rule {self:?} passed for {}: {passed}", path.display());
        passed
    }
}

#[instrument(level = "TRACE")]
pub(super) fn meta(path: &Path) -> RuleResult<Metadata> {
    path.metadata()
        .inspect(|m| trace!("Metadata acquired for {}: {:?}", path.display(), m))
        .inspect_err(|_err| error!("Failed to get metadata for {}", path.display()))
        .map_err(|err| RuleError::MetadataError(err, path.to_path_buf()))
}
