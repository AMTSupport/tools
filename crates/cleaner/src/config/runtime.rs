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
use anyhow::{Context, Error};
use clap::Parser;
use lib::anyhow::Result;
use std::sync::RwLock;
use tracing::dispatcher::DefaultGuard;
use tracing::error;
use tracing_subscriber::util::SubscriberInitExt;

#[derive(Debug)]
pub struct Runtime {
    pub cli: Cli,
    pub errors: RwLock<Vec<Error>>,
    pub logger: DefaultGuard,
}

impl Runtime
where
    Self: Send + Sync + 'static,
{
    pub fn new() -> Result<Self> {
        let cli = Cli::try_parse().context("Failed to parse CLI arguments")?;
        let errors = RwLock::new(Vec::new());
        let logger = tracing_subscriber::fmt()
            .with_max_level(tracing::Level::TRACE)
            .without_time()
            .pretty()
            .finish()
            .set_default();

        Ok(Self { cli, errors, logger })
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
