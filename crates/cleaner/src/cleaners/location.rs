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

use glob::Paths;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use tracing::{debug, error, instrument, trace, warn};

#[derive(Debug, Clone)]
pub enum Location {
    Environment(String),
    Globbing(String),
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

    pub fn get_recursed(&self) -> HashSet<PathBuf> {
        recurse(self.get_path())
    }
}

#[instrument(skip(top))]
fn recurse(top: Vec<PathBuf>) -> HashSet<PathBuf> {
    let mut paths = HashSet::new();

    for path in top {
        trace!("Parsing path {}", path.display());

        if path.is_dir() {
            trace!("Path {} is a directory; recursing into.", path.display());
            let sub_files = match path.read_dir() {
                Ok(files) => files,
                Err(_) => {
                    error!("Failed to read directory {}", path.display());
                    continue;
                }
            }
            .flatten()
            .map(|entry| entry.path())
            .collect::<Vec<_>>();

            paths.extend(&mut recurse(sub_files).into_iter());
        } else {
            trace!("Path {} is a file; appending", path.display());
            paths.extend_one(path);
        }
    }

    paths
}

#[instrument]
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

#[instrument]
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
    .expect(&*format!("Pattern error in globbing {pattern}"))
}

#[instrument]
fn sub(location: &Location, sub_location: &String) -> Vec<PathBuf> {
    debug!("Subbing {} with {}", location.get_path().len(), sub_location);

    let parents = location.get_path();
    let mut paths = Vec::new();

    for parent in &parents {
        let mut subs = globbing(&*parent.join(&sub_location).display().to_string()).into_iter();
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
