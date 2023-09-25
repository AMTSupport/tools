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

use crate::app::App;
use crate::ui::cli::action::Action;
use anyhow::Result;
use std::fmt::Debug;
use std::path::Path;
use tracing::instrument;

#[derive(Debug)]
pub struct CliUI {
    app: App,
}

impl CliUI {
    #[instrument(level = "TRACE")]
    pub fn new(initial_action: Action, destination: Option<&Path>) -> Result<Self> {
        let runtime = initial_action.prepare(destination)?;
        let app = App::new(runtime);

        Ok(Self { app })
    }

    #[instrument(level = "TRACE")]
    pub async fn run(&mut self, event: Action) -> Result<()> {
        self.app.running = true;

        event.run(&mut self.app.runtime).await?;
        // TODO :: catch errors and save config before exiting
        self.app.runtime.config.save().await?;

        self.app.running = false;

        Ok(())
    }
}
