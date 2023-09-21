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
use crate::ui::{Ui, UiBuidableFiller, UiBuildable};
use anyhow::Result;
use inquire::Text;
use std::fmt::Debug;
use std::path::Path;
use tracing::{error, instrument, trace};

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

impl Ui for CliUI {}

impl UiBuidableFiller for CliUI {
    #[instrument(level = "TRACE")]
    async fn fill<B: UiBuildable<V>, V: From<B> + Debug>() -> Result<V> {
        let mut builder = B::default();
        let mut required_values = B::REQUIRED_FIELDS.to_vec();
        let mut optional_values = B::OPTIONAL_FIELDS.to_vec();

        for env_filled in builder.filled_fields() {
            trace!("Field {env_filled} was filled from env");
            required_values.retain(|field| field != env_filled);
            optional_values.retain(|field| field != env_filled);
        }

        for field in required_values {
            let message = format!("Please enter the value for {field}");
            let prompt = Text::new(&*message)
                .with_help_message("This value is required")
                .with_placeholder("Enter value here...");

            let value = prompt.prompt()?;
            builder.set_field(field, &*value)?;
        }

        for field in optional_values {
            let message = format!("Please enter the value for {field}");
            let prompt = Text::new(&*message).with_help_message("This value is optional").with_default("");

            let value = prompt.prompt()?;
            builder.set_field(field, &*value)?;
        }

        builder.build()
    }

    #[instrument(level = "TRACE")]
    async fn modify<B: UiBuildable<V>, V: From<B> + Debug>(mut builder: B) -> Result<V> {
        for field in B::REQUIRED_FIELDS {
            let current = builder.display(field);
            let message = format!("Please enter the value for {field}");
            let prompt = Text::new(&*message)
                .with_help_message("This value is required")
                .with_placeholder("Enter value here...")
                .with_default(&*current);

            match prompt.prompt() {
                Ok(value) => builder.set_field(field, &*value)?,
                Err(err) => {
                    error!("Failed to prompt for field {field}: {err}");
                    error!("Using current value: {current}");
                }
            }
        }

        for field in B::OPTIONAL_FIELDS {
            let current = builder.display(field);
            let message = format!("Please enter the value for {field}");
            let prompt = Text::new(&*message)
                .with_help_message("This value is optional")
                .with_placeholder("Enter value here...")
                .with_default(&*current);

            match prompt.prompt() {
                Ok(value) => builder.set_field(field, &*value)?,
                Err(err) => {
                    error!("Failed to prompt for field {field}: {err}");
                    error!("Using current value: {current}");
                }
            }
        }

        builder.build()
    }
}
