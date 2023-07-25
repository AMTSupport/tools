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

use anyhow::Result;
use std::env;
use std::path::PathBuf;
use thiserror::Error;
use tracing::{error, instrument, trace};

// pub mod builder;
pub mod cleaner;
// pub mod env_cleaner;
pub mod impls;
pub mod location;

#[derive(Error, Debug)]
pub enum CleanerError {
    #[error("Issue with Environment variable {0} while {1}")]
    EnvError(String, String),

    #[error("Issue with paths {0} while {1}")]
    PathError(String, String),
}

#[instrument]
fn env_path(env: String) -> Result<PathBuf> {
    env::var(&env)
        .inspect(|env_var| trace!("Using environment variable: {}", env_var))
        .map_err(|_e| CleanerError::EnvError(env.to_owned(), format!("Getting value from {env}")).into())
        .map(|var| PathBuf::from(var))
}

#[instrument]
fn env_dir<'l>(env: String) -> Result<PathBuf> {
    let path = env_path(env.clone())?;

    match path.is_dir() {
        true => Ok(path),
        false => {
            Err(CleanerError::PathError(env.to_owned(), format!("Path {} is not a directory", path.display())).into())
        }
    }
}
