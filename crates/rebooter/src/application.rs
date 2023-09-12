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

use crate::runtime::Runtime;
use anyhow::Result;
use clap::Parser;
use lib::cli::Flags;
use thiserror::Error;
use tracing::{instrument, Instrument};
use lib::sysexits::ExitCode;

#[derive(Debug, Error)]
pub enum Error {
    #[error("error during cli parsing")]
    CliError(#[from] clap::Error),

    #[error("superuser privileges are required; Code {0}")]
    PrivilegeError(ExitCode),
}

#[derive(Debug, Clone, Copy, Parser)]
pub struct Cli {
    #[command(flatten)]
    pub flags: Flags,
}

#[instrument(ret, err)]
pub async fn run(runtime: Runtime) -> Result<ExitCode> {
    if let Some(code) = lib::helper::require_elevated_privileges() {
        return Err(Error::PrivilegeError(code).into());
    }

    let requires_reboot = super::requires_restart();

    Ok(ExitCode::Ok)
}
