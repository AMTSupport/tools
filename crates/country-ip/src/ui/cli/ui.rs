/*
 * Copyright (C) 2024. James Draycott me@racci.dev
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, version 3.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.
 * See the GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program. If not, see https://www.gnu.org/licenses/.
 */

use super::oneshot::OneshotAction;
use amt_lib::populate;
use amt_lib::ui::cli::error::CliError;
use amt_lib::ui::cli::flags::CommonFlags;
use amt_lib::ui::cli::oneshot::OneshotHandler;
use amt_lib::ui::cli::{CliResult, CliUi};
use amt_lib::ui::Ui;
use tracing::{error, info, info_span};
use tracing_appender::non_blocking::WorkerGuard;
use tracing_indicatif::span_ext::IndicatifSpanExt;

#[derive(Default, Debug)]
pub struct CountryIPCli {
    _guard: Option<WorkerGuard>,
}

impl CliUi for CountryIPCli {}

impl Ui for CountryIPCli {
    type Args = ();

    fn new(_args: Self::Args) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        Ok(Self { _guard: None })
    }
}

impl OneshotHandler for CountryIPCli {
    type OneshotAction = OneshotAction;

    async fn handle(&mut self, command: Self::OneshotAction, flags: &CommonFlags) -> CliResult<()> {
        populate!(self, flags);

        let span = info_span!("feedback");
        span.pb_set_style(&amt_lib::ui::cli::progress::style_spinner());
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
