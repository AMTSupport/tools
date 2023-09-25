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
#[cfg(feature = "ui-repl")]
use super::repl::ReplAction;
use lib::cli::Flags as CommonFlags;
use lib::ui::cli::cli::{AsyncCliUI, CliResult, CliUI};
use lib::ui::cli::error::CliError;
use tracing::{error, info, info_span, instrument};
use tracing_appender::non_blocking::WorkerGuard;
use tracing_indicatif::span_ext::IndicatifSpanExt;

#[derive(Debug)]
pub struct CountryIPCli {
    _guard: Option<WorkerGuard>,
}

impl CliUI for CountryIPCli {
    type OneShotCommand = OneshotAction;
    #[cfg(feature = "ui-repl")]
    type ReplCommand = ReplAction;

    fn new(_args: Self::Args) -> CliResult<Self>
    where
        Self: Sized,
    {
        Ok(Self { _guard: None })
    }
}

impl AsyncCliUI for CountryIPCli {
    #[instrument(skip(self))]
    async fn handle_command(&mut self, command: Self::OneShotCommand, flags: &CommonFlags) -> CliResult<()> {
        if self._guard.is_none() {
            self._guard = Some(lib::log::init(env!("CARGO_PKG_NAME"), flags.verbose));
        }

        let span = info_span!("feedback");
        span.pb_set_style(&lib::ui::cli::progress::style_spinner());
        span.pb_start();

        match command {
            OneshotAction::Get { country, ipv6 } => {
                span.pb_set_message("Fetching country data...");
                let country = crate::get_country(&country).map_err(|err| CliError::Source(err.into()))?;

                span.pb_set_message("Getting IP Records...");
                let random = crate::get(&country, &ipv6).await.map_err(CliError::Source)?;

                info!("{} => {random}", country.iso_short_name());
            }
            OneshotAction::Lookup { addr, .. } => {
                span.pb_set_message("Looking up IP address...");
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

    #[cfg(feature = "ui-repl")]
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
