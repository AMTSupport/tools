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

use crate::config::config::Config;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Runtime {
    /// The directory which contains the settings.json file,
    /// and the backup root directory.
    pub directory: PathBuf,

    /// The configuration for the backup(s).
    pub(crate) config: Config,
}

impl Runtime {
    pub(crate) fn wrapping(backup_directory: PathBuf) -> Self {
        Self {
            config: Config {
                path: Some(backup_directory.join(Config::FILENAME)),
                ..Default::default()
            },
            directory: backup_directory,
        }
    }
}

impl From<Runtime> for PathBuf {
    fn from(ref value: Runtime) -> Self {
        value.directory.clone()
    }
}
