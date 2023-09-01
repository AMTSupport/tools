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

use crate::application::{Cli, Error};
use anyhow::Result;
use clap::Parser;
use tracing::info;

#[derive(Debug, Clone, Copy)]
pub struct Runtime {
    pub cli: Cli,
}

impl Runtime {
    pub async fn new() -> Result<Self> {
        let cli = match Cli::try_parse() {
            Ok(cli) => cli,
            Err(err) => return Err(Error::CliError(err).into()),
        };

        let _ = lib::log::init(env!("CARGO_CRATE_NAME"), cli.flags.verbose);

        if cli.flags.dry_run {
            info!("Dry run enabled, no actions will be taken");
        }

        Ok(Self { cli })
    }
}
