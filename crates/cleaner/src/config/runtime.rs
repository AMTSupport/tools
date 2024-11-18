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

use super::cli::Cli;
use anyhow::Error;
use anyhow::Result;
use std::sync::RwLock;
use tracing::error;

#[derive(Debug)]
pub struct Runtime {
    pub cli: Cli,
    pub errors: RwLock<Vec<Error>>,
}

impl Runtime
where
    Self: Send + Sync + 'static,
{
    pub fn new() -> Result<Self> {
        use clap::Parser;

        let cli = Cli::parse();
        let errors = RwLock::new(Vec::new());

        Ok(Self { cli, errors })
    }

    pub fn submit_error(&mut self, error: Error) {
        if self.cli.flags.verbose > 0 {
            error!("Error: {}", error);
        }

        let mut write = self.errors.write().unwrap();
        write.push(error);
        drop(write);
    }
}
