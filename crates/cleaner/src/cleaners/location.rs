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

use glob::Paths;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::exit;
use tracing::{debug, error, instrument, trace, warn};

#[derive(Debug, Clone)]
pub enum Location {
    Environment(String),

    /// A globbing collection of paths.
    ///
    /// This can be used to specify a glob such as `C:\Users\*\AppData\Local\Temp\*`.
    /// Or to specify a full path such as `/home/racci/.cache/`.
    Globbing(String),

    /// A sub-location of another location.
    ///
    /// This can be used to specify a location which is determined from the contents of another location.
    Sub(&'static Location, String),
}

impl Location {
    pub fn get_path(&self) -> Vec<PathBuf> {
        match self {
            Location::Environment(var) => environment(var),
            Location::Globbing(pattern) => globbing(pattern).flatten().collect(),
            Location::Sub(location, sub_location) => sub(location, sub_location).into_iter().collect(),
        }
        .into_iter()
        .collect()
    }

    pub fn get_recursed(&self) -> Vec<PathBuf> {
        let init_top = self.get_path();
        recurse(init_top)
    }
}

#[instrument(level = "TRACE", skip(top_init))]
fn recurse(top_init: Vec<PathBuf>) -> Vec<PathBuf> {
    let mut collection = HashMap::new();
    fn recurser(top: Vec<PathBuf>, map: &mut HashMap<PathBuf, Vec<PathBuf>>) {
        if top.is_empty() {
            return;
        }

        let iter = top.into_iter();
        for path in iter {
            if !path.exists() {
                warn!("Path {} does not exist", path.display());
                continue;
            }

            if path.is_symlink() {
                warn!("Path {} is a symlink; we don't dare touch these!", path.display());
                continue;
            }

            if !path.is_dir() {
                error!("Path {} is not a directory; this shouldn't happen!", path.display());
                continue;
            }

            if map.contains_key(&path) {
                trace!("Path {} has already been recursed into; skipping", path.display());
                continue;
            }

            if let Ok(files) = fs::read_dir(&path) {
                let (sub_dirs, sub_files) = files.filter_map(|e| e.ok().map(|e| e.path())).partition(|p| p.is_dir());
                map.insert(path, sub_files);
                recurser(sub_dirs, map);
            } else {
                warn!("Failed to read directory {}", path.display());
            }
        }
    }

    let mut recursing_paths = Vec::new();
    let top_iter = top_init.into_iter();
    for path in top_iter {
        if !path.exists() {
            warn!("Path {} does not exist", path.display());
            continue;
        }

        if path.is_file() || path.is_symlink() {
            let parent = path.parent().unwrap_or_else(|| {
                error!("Path {} has no parent; this shouldn't happen!", path.display());
                exit(1)
            });

            if !collection.contains_key(parent) {
                collection.insert(parent.to_path_buf(), Vec::new());
            }

            // SAFETY: We just inserted the parent into the collection.
            collection.get_mut(parent).unwrap().push(path.to_path_buf());
            continue;
        }

        if path.is_symlink() {
            trace!("Path {} is a symlink; treating as file.", path.display());
            continue;
        }

        if path.is_dir() {
            recursing_paths.push(path);
        }
    }

    recurser(recursing_paths, &mut collection);
    collection.into_iter().flat_map(|(_, v)| v).collect()
}

#[instrument(level = "TRACE")]
fn environment(var: &str) -> Vec<PathBuf> {
    if let Ok(path) = std::env::var(var) {
        if Path::new(&path).exists() {
            trace!("Environment variable {} is set to {}", var, path);
            vec![PathBuf::from(path)]
        } else {
            warn!("Environment variable {} is set to a non-existent path: {}", var, path);
            vec![]
        }
    } else {
        trace!("Environment variable {} is not set", var);
        vec![]
    }
}

#[instrument(level = "TRACE")]
fn globbing(pattern: &str) -> Paths {
    debug!("Globbing pattern {}", pattern);

    glob::glob_with(
        pattern,
        glob::MatchOptions {
            case_sensitive: true,
            require_literal_separator: true,
            require_literal_leading_dot: false,
        },
    )
    .unwrap_or_else(|_| panic!("Pattern error in globbing {pattern}"))
}

#[instrument(level = "TRACE")]
fn sub(location: &Location, sub_location: &String) -> Vec<PathBuf> {
    debug!("Subbing {} with {}", location.get_path().len(), sub_location);

    let parents = location.get_path();
    let mut paths = Vec::new();

    for parent in &parents {
        let mut subs = globbing(&parent.join(sub_location).display().to_string());
        while let Some(Ok(sub)) = subs.next() {
            debug!("Subbed {} with {}", parent.display(), sub.display());
            match sub.exists() {
                true => {
                    trace!("Path {} exists", sub.display());
                    paths.push(sub);
                }
                false => warn!("Path {} does not exist", sub.display()),
            }
        }
    }

    paths
}
