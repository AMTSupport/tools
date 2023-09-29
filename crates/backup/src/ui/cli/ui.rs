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

use crate::config::runtime::Runtime;
use crate::ui::cli::action::Action;
use anyhow::Result;
use lib::cli::Flags as CommonFlags;
use lib::ui::cli::oneshot::OneshotHandler;
use lib::ui::cli::{CliResult, CliUi};
use lib::ui::Ui;
use tracing_appender::non_blocking::WorkerGuard;

pub struct BackupCli {
    runtime: Option<Runtime>,
    _gaurd: Option<WorkerGuard>,
}

impl CliUi for BackupCli {}

impl OneshotHandler for BackupCli {
    type Action = Action;

    async fn handle(&mut self, command: Self::Action, flags: &CommonFlags) -> CliResult<()> {
        if self._gaurd.is_none() {
            self._gaurd = Some(lib::log::init(env!["CARGO_PKG_NAME"], flags.verbose));
        }

        if self.runtime.is_none() {
            self.runtime = Some(command.prepare()?);
        }

        command.run(self.runtime.as_mut().unwrap()).await?;

        Ok(())
    }
}

impl Ui for BackupCli {
    fn new(_args: Self::Args) -> Result<Self>
    where
        Self: Sized,
    {
        Ok(Self {
            runtime: None,
            _gaurd: None,
        })
    }
}
