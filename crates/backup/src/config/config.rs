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

use crate::config::backend::Backend;
use crate::config::rules::Rules;
use anyhow::Result;
use futures_util::TryFutureExt;
use std::io;
use std::path::{Path, PathBuf};
use tokio::fs;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tracing::{debug, error, instrument, trace};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Unable to find configuration file at {0}")]
    NotFound(PathBuf),

    #[error("Failed to read/write configuration file, check permissions?")]
    IoFile(#[source] io::Error),

    #[error("Failed to create configuration directory, check permissions?")]
    IoDir(#[source] io::Error),

    #[error(
        r#"
        Failed to (de)serialise configuration file.
        Please ensure you have not manually modified the configuration file.
        If the problem persists you may need to delete the configuration file and re-create it.
    "#
    )]
    Serde(#[source] serde_json::Error),

    #[error(r#"
        Failed to find configuration file, please run `backup init` to create a new configuration.
        If you already have a configuration, please set BACKUP_CONFIG or enter the directory containing the configuration file.
    "#)]
    Find,
}

#[derive(Default, Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Config {
    /// Whether the configuration has been mutated.
    /// This is used to determine whether to save the configuration.
    /// If the configuration has not been mutated, it will not be saved.
    /// This is to prevent unnecessary writes to the configuration file.
    pub(crate) mutated: bool,

    pub rules: Rules,
    pub exporters: Vec<Backend>,

    #[serde(skip)]
    pub path: Option<PathBuf>,
}

impl Config {
    pub const FILENAME: &'static str = "settings.json";
    pub const ENV_VAR: &'static str = concat!(env!("CARGO_PKG_NAME"), "_CONFIG");

    pub fn new(parent_directory: &Path) -> Self {
        Self {
            mutated: true,
            rules: Rules::default(),
            exporters: vec![],
            path: Some(parent_directory.join(Self::FILENAME)),
        }
    }

    /// Attempts to find a configuration file.
    /// and returns an error if it cannot be found.
    ///
    /// The returned path is the path to the configuration file.
    ///
    /// This will search for the following files in order:
    /// - $BACKUP_CONFIG
    /// - $PWD/settings.json
    #[instrument]
    pub fn find(directory: Option<&Path>) -> Result<PathBuf> {
        use std::env;

        env::var(Self::ENV_VAR)
            .map(PathBuf::from)
            .map_err(|_| Error::Find)
            .and_then(|dir| {
                let path = dir.join(Self::FILENAME);
                path.exists().then(|| path).ok_or(Error::Find)
            })
            .or_else(|_| {
                let path = directory.map(|p| p.join(Self::FILENAME));
                path.as_deref().is_some_and(Path::exists).then(|| path.unwrap()).ok_or(Error::Find)
            })
            .or_else(|_| {
                let path = env::current_dir().map(|p| p.join(Self::FILENAME));
                path.as_deref().is_ok_and(Path::exists).then(|| path.unwrap()).ok_or(Error::Find)
            })
            .map_err(Into::into)
    }

    #[instrument]
    pub async fn load(path: &Path) -> Result<Self> {
        match path.exists() {
            false => Err(Error::NotFound(path.to_path_buf()).into()),
            true => {
                let mut slice = vec![];
                fs::File::open(&*path).await?.read(&mut slice).await?;
                serde_json::from_slice(&*slice).map_err(Error::Serde).and_then(|mut config: Config| {
                    config.path.replace(path.into());
                    Ok(config)
                }).map_err(Into::into)
            }
        }
    }

    /// Saves the configuration to the given directory.
    /// This will overwrite any existing configuration.
    ///
    /// If the directory does not exist, it will be created.
    ///
    /// This will create a file called `settings.json` in the given directory.
    /// If the file already exists, and the `mutated` flag is not set, this will not write to the file.
    #[instrument]
    pub async fn save(&self) -> Result<()> {
        let path = match self.path {
            None => {
                return Err(Error::IoFile(io::Error::new(
                    io::ErrorKind::NotFound,
                    "Config is missing path to file.",
                ))
                .into())
            }
            Some(ref path) => path,
        };

        match path.parent() {
            None => {
                return Err(
                    Error::IoDir(io::Error::new(io::ErrorKind::Other, "Unable to get parent directory.")).into(),
                )
            }
            Some(parent) => {
                if !parent.exists() {
                    fs::create_dir_all(parent).await.map_err(Error::IoDir)?
                }
            }
        }

        if path.exists() && !self.mutated {
            debug!("Not saving config as it has not been mutated");
            return Ok(());
        }

        trace!("Saving config to {}", path.display());
        match fs::File::create(path).await.map_err(Error::IoFile) {
            Err(err) => {
                error!("Failed to create config file: {err}");
                Err(err.into())
            }
            Ok(mut file) => {
                let serde = serde_json::to_vec_pretty(&self).map_err(Error::Serde)?;
                file.write_all(serde.as_slice()).map_err(Error::IoFile).await?;
                Ok(())
            }
        }
    }
}
