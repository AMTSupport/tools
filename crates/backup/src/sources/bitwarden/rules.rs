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

use super::BitWardenCore;
use crate::config::runtime::Runtime;
use crate::sources::auto_prune::Prune;
use anyhow::{Context, Result};
use std::path::PathBuf;

impl Prune for BitWardenCore {
    fn files(&self, config: &Runtime) -> Result<Vec<PathBuf>> {
        use std::path::MAIN_SEPARATOR;

        let glob = format!(
            "{root}{MAIN_SEPARATOR}backup-{org}/*.json",
            root = &config.directory.display(),
            org = &self.org_name
        );

        glob::glob(&glob)
            .with_context(|| format!("Glob backup files for {glob}"))
            .map(|g| g.flatten().collect())
    }
}
