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
use lib::cli::Flags as CommonFlags;
use lib::ui::cli::error::CliError;
use lib::ui::cli::oneshot::OneshotHandler;
use lib::ui::cli::{CliResult, CliUi};
use tracing::{error, info, info_span};
use tracing_appender::non_blocking::WorkerGuard;
use tracing_indicatif::span_ext::IndicatifSpanExt;

#[derive(Default, Debug)]
pub struct CountryIPCli {
    _guard: Option<WorkerGuard>,
}

impl CliUi for CountryIPCli {}

impl OneshotHandler for CountryIPCli {
    type Action = OneshotAction;

    async fn handle(&mut self, command: Self::Action, flags: &CommonFlags) -> CliResult<()> {
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
}
