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

use crate::ui::actions::Action;
use lib::cli::Flags as CommonFlags;
use lib::populate;
use lib::ui::cli::oneshot::OneshotHandler;
use lib::ui::cli::{CliResult, CliUi};
use lib::ui::Ui;
use tracing::info;
use tracing_appender::non_blocking::WorkerGuard;

pub struct RebooterCli {
    _guard: Option<WorkerGuard>,
}

impl Ui for RebooterCli {
    fn new(args: Self::Args) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        todo!()
    }
}

impl CliUi for RebooterCli {}

impl OneshotHandler for RebooterCli {
    type Action = Action;

    async fn handle(&mut self, command: Self::Action, flags: &CommonFlags) -> CliResult<()> {
        populate!(self, flags);

        match command {
            Action::Test(reason) => match reason.valid() {
                true => info!("{reason} would require a restart."),
                false => info!("{reason} would not require a restart."),
            },
        }

        Ok(())
    }
}
