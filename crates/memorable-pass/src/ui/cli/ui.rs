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

use crate::config;
use crate::rules::Rules;
use crate::ui::cli::action::Action;
use amt_lib::ui::cli::flags::{CommonFlags, OutputFormat};
use amt_lib::ui::cli::oneshot::OneshotHandler;
use amt_lib::ui::cli::{CliResult, CliUi};
use amt_lib::ui::Ui;
use serde_json::json;
use tokio::runtime::Handle;
use tracing::{info, instrument};
use tracing_appender::non_blocking::WorkerGuard;

#[derive(Debug)]
pub struct MemorablePassCli {
    _guard: Option<WorkerGuard>,
    rules: Option<Rules>,
}

impl CliUi for MemorablePassCli {}

impl OneshotHandler for MemorablePassCli {
    type OneshotAction = Action;

    #[instrument(level = "TRACE", skip(self))]
    async fn handle(&mut self, command: Self::OneshotAction, flags: &CommonFlags) -> CliResult<()> {
        if self._guard.is_none() {
            self._guard = Some(amt_lib::log::init(env!("CARGO_PKG_NAME"), flags));
        }

        match command {
            Action::Generate { rules } => {
                let passwords = crate::generate(&rules).await;
                self.rules.replace(rules);

                match flags.format {
                    OutputFormat::Human => {
                        info!("Generated passwords:\n{passwords}", passwords = passwords.join("\n"));
                    }
                    OutputFormat::Json => {
                        let json = json! {
                            {
                                "passwords": passwords,
                                "rules": self.rules
                            }
                        };

                        println!("{json:#}");
                    }
                }
            }
        }

        Ok(())
    }
}

impl Ui<CliResult<Self>> for MemorablePassCli {
    fn new(_args: Self::Args) -> CliResult<Self>
    where
        Self: Sized,
    {
        // Preload the words
        Handle::current().spawn(async {
            let _preload = &config::asset::WORDS;
        });

        Ok(Self {
            _guard: None,
            rules: None,
        })
    }
}
