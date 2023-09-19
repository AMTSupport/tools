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

use super::oneshot::OneshotAction;
use super::repl::ReplAction;
use lib::cli::Flags as CommonFlags;
use lib::ui::cli::cli::{AsyncCliUI, CliResult, CliUI};
use lib::ui::cli::error::CliError;
use tracing::{error, info};
use tracing_appender::non_blocking::WorkerGuard;

#[derive(Debug)]
pub struct CountryIPCli {
    _guard: Option<WorkerGuard>,
}

impl CliUI for CountryIPCli {
    type OneShotCommand = OneshotAction;
    type ReplCommand = ReplAction;

    fn new(_args: Self::Args) -> CliResult<Self>
    where
        Self: Sized,
    {
        return Ok(Self { _guard: None });
    }
}

impl AsyncCliUI for CountryIPCli {
    async fn handle_command(&mut self, command: Self::OneShotCommand, flags: &CommonFlags) -> CliResult<()> {
        if self._guard.is_none() {
            self._guard = Some(lib::log::init(env!("CARGO_PKG_NAME"), flags.verbose));
        }

        match command {
            OneshotAction::Get { country, ipv6 } => {
                let country = crate::get_country(&country).map_err(|err| CliError::Source(err.into()))?;
                let random = crate::get(&country, &ipv6).await.map_err(|err| CliError::Source(err.into()))?;
                info!(
                    "Generated random IP address for {} => {random}",
                    country.iso_short_name()
                );
            }
            OneshotAction::Lookup { addr, .. } => {
                let ip = crate::lookup(&addr).await;

                match ip {
                    Ok(country) => info!("{addr} => {}", country.iso_short_name()),
                    Err(err) => {
                        error!("Failed to lookup IP address: {err}");
                        info!("{addr} => Unknown")
                    }
                }
            }
        }

        Ok(())
    }

    async fn handle_repl_command(&mut self, command: Self::ReplCommand, flags: &CommonFlags) -> CliResult<bool> {
        match command {
            ReplAction::Quit => Ok(true),
            ReplAction::Get { country, ipv6 } => {
                self.handle_command(OneshotAction::Get { country, ipv6 }, flags).await?;
                Ok(false)
            }
            ReplAction::Lookup { addr } => {
                self.handle_command(OneshotAction::Lookup { addr }, flags).await?;
                Ok(false)
            }
        }
    }
}
