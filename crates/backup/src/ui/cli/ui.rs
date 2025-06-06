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

use crate::config::runtime::Runtime;
use crate::ui::cli::action::Action;
use amt_lib::ui::cli::flags::CommonFlags;
use amt_lib::ui::cli::oneshot::OneshotHandler;
use amt_lib::ui::cli::{CliResult, CliUi};
use amt_lib::ui::Ui;
use anyhow::Result;
use tracing_appender::non_blocking::WorkerGuard;

#[derive(Debug)]
pub struct BackupCli {
    pub runtime: Option<Runtime>,
    _guard: Option<WorkerGuard>,
}

impl CliUi for BackupCli {}

impl OneshotHandler for BackupCli {
    type OneshotAction = Action;

    async fn handle(&mut self, command: Self::OneshotAction, flags: &CommonFlags) -> CliResult<()> {
        if self._guard.is_none() {
            self._guard = Some(amt_lib::log::init(env!["CARGO_PKG_NAME"], flags));
        }

        if self.runtime.is_none() {
            self.runtime = Some(command.initialise().await?);
        }

        command.run(self).await?;

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
            _guard: None,
        })
    }
}
