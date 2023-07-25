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
use std::path::{Path, PathBuf};
use tracing::{trace, warn};

#[derive(Debug, Clone)]
pub enum Location {
    Environment(String),
    Globbing(String),
    Relative(String),
    Sub(&'static Location, String),
}

impl Location {
    pub fn get_path(&self) -> Vec<PathBuf> {
        match self {
            Location::Environment(var) => environment(var).into_iter().collect(),
            Location::Globbing(pattern) => globbing(pattern).flatten().collect(),
            Location::Relative(path) => relative(path).into_iter().collect(),
            Location::Sub(location, sub_location) => sub(location, sub_location).into_iter().collect(),
        }
    }
}

fn environment(var: &str) -> Option<PathBuf> {
    if let Ok(path) = std::env::var(var) {
        if Path::new(&path).exists() {
            trace!("Environment variable {} is set to {}", var, path);
            Some(PathBuf::from(path))
        } else {
            warn!("Environment variable {} is set to a non-existent path: {}", var, path);
            None
        }
    } else {
        trace!("Environment variable {} is not set", var);
        None
    }
}

fn globbing(pattern: &str) -> Paths {
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

fn relative(path: &str) -> Option<PathBuf> {
    if Path::new(path).exists() {
        trace!("Path {} exists", path);
        Some(PathBuf::from(path))
    } else {
        warn!("Path {} does not exist", path);
        None
    }
}

fn sub(location: &Location, sub_location: &String) -> Vec<PathBuf> {
    let parents = location.get_path();
    let mut paths = Vec::new();

    for parent in &parents {
        let mut subs = globbing(&*parent.join(&sub_location).display().to_string());
        while let Some(Ok(sub)) = subs.next() {
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
