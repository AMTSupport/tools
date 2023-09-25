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

use crate::config;
use crate::rules::rules::Rules;
use crate::ui::cli::action::Action;
use lib::cli::Flags as CommonFlags;
use lib::ui::cli::cli::{AsyncCliUI, CliResult, CliUI};
use tokio::runtime::Handle;
use tracing::{info, instrument};
use tracing_appender::non_blocking::WorkerGuard;

#[derive(Debug)]
pub struct MemorablePassCli {
    _guard: Option<WorkerGuard>,
    rules: Option<Rules>,
}

impl CliUI for MemorablePassCli {
    type OneShotCommand = Action;

    fn new(_args: Self::Args) -> CliResult<Self>
    where
        Self: Sized,
    {
        // Preload the words
        Handle::current().spawn(async {
            let _preload = &config::asset::WORDS;
        });

        return Ok(Self {
            _guard: None,
            rules: None,
        });
    }
}

impl AsyncCliUI for MemorablePassCli {
    #[instrument(level = "TRACE", skip(self))]
    async fn handle_command(&mut self, command: Self::OneShotCommand, flags: &CommonFlags) -> CliResult<()> {
        if self._guard.is_none() {
            self._guard = Some(lib::log::init(env!("CARGO_PKG_NAME"), flags.verbose));
        }

        match command {
            Action::Generate { rules } => {
                let passwords = crate::generate(&rules).await;
                self.rules.replace(rules);

                info!(
                    "Generated passwords:\n{passwords}",
                    passwords = passwords.join("\n")
                );
            }
        }

        Ok(())
    }

    #[cfg(feature = "ui-repl")]
    #[instrument(level = "TRACE", skip(self))]
    async fn handle_repl_command(&mut self, command: Self::ReplCommand, flags: &CommonFlags) -> CliResult<bool> {
        match command.action {
            ReplCommand::Generate => {
                let rules = self.rules.get_or_insert_with(|| {
                    debug!("No rules set, using defaults");
                    Rules::default()
                });

                let passwords = generate(&rules).await;
                info!(
                    "Generated passwords:\n\n{passwords}\n",
                    passwords = passwords.join("\n")
                );
            }
            ReplCommand::Rules(rules) => {
                let previous_rules = self.rules.replace(rules);

                if let Some(previous_rules) = previous_rules {
                    debug!("Replacing previous rules.");
                    debug!("Previous rules:\n\n{previous_rules:?}\n");
                }
            }
            ReplCommand::Ping => {
                info!("Pong!");
            }
            ReplCommand::Quit => {
                info!("Quitting...");
                return Ok(true);
            }
        }

        Ok(false)
    }
}
