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

use crate::ui::cli::error::CliError;
use crate::ui::{UiBuidableFiller, UiBuildable};
use anyhow::Result;
use clap::{CommandFactory, Parser};
use inquire::Text;
use std::fmt::Debug;
use std::io::{stdout, Write};
use tracing::{debug, error, instrument, trace};

pub type CliResult<T> = Result<T, CliError>;

pub trait AsyncCliUI: CliUI + Send + Sync + 'static {
    async fn handle_command(&mut self, command: Self::OneShotCommand) -> CliResult<()>;

    async fn handle_repl_command(&mut self, command: Self::ReplCommand) -> CliResult<bool>;

    /// Run the CLI Application in a one-shot mode
    async fn run(&mut self) -> CliResult<()> {
        let command = Self::OneShotCommand::parse();
        match self.handle_command(command).await {
            Ok(_) => Ok(()),
            Err(err) => {
                error!("Failed to handle command:\n{err}");
                Err(err)
            }
        }
    }

    /// Run in REPL mode (Read-Eval-Print-Loop)
    ///
    /// This is a mode where the user can enter commands and have them executed
    /// in a loop until they exit.
    async fn repl(&mut self) -> CliResult<()> {
        loop {
            let line = readline()?;
            let line = line.trim();

            debug!("Read line: {line}");

            match line {
                l if l.is_empty() | l.starts_with('#') => continue,
                l => match self.respond(l).await {
                    Ok(true) => break,
                    Ok(false) => continue,
                    Err(err) => {
                        error!(
                            r#"
                            There was an error while processing your command:
                            ----
                            {err}
                            ----
                            Failed to respond to line {l}
                            "#
                        );
                        continue;
                    }
                },
            }
        }

        Ok(())
    }

    async fn respond(&mut self, line: &str) -> CliResult<bool> {
        let args = shlex::split(line).ok_or_else(|| CliError::ParseError(line.into()))?;
        let mut matches = Self::ReplCommand::command().try_get_matches_from(&args).map_err(CliError::InvalidCommand)?;

        match matches.subcommand() {
            // Some(("ping", _)) => {
            //     writeln!(stdout(), "pong").map_err(CliError::WriteError)?;
            //     stdout().flush().map_err(CliError::WriteError)?;
            //     Ok(false)
            // }
            // Some(("quit", _)) | Some(("q", _)) => {
            //     writeln!(stdout(), "Goodbye!").map_err(CliError::WriteError)?;
            //     stdout().flush().map_err(CliError::WriteError)?;
            //     Ok(true)
            // }
            // Some(("help", _)) | Some(("h", _)) => {
            //     Self::ReplCommand::command().print_long_help().map_err(CliError::WriteError)?;
            //     Ok(false)
            // }
            _ => {
                use clap::FromArgMatches;

                let parsed = Self::ReplCommand::from_arg_matches_mut(&mut matches).map_err(CliError::InvalidCommand)?;
                self.handle_repl_command(parsed).await?;
                Ok(false)
            }
        }
    }
}

pub trait CliUI {
    /// The command that will be used in one-shot mode
    type OneShotCommand: Parser;

    /// The command that will be used to parse REPL commands
    type ReplCommand: Parser;

    /// The arguments that will be used to create the CLI UI
    ///
    /// This can be used to pass in configuration options, or other data.
    /// This is not required to be used, but is available if needed.
    type Args = ();

    /// Create a new instance of the CLI UI
    ///
    /// This is used to create a new instance of the CLI UI, and can be used to
    /// parse arguments, or other data.
    ///
    /// Your logging guard should be set up here, as well as any other
    /// configuration that is needed.
    fn new(args: Self::Args) -> CliResult<Self>
    where
        Self: Sized;
}

fn readline() -> CliResult<String> {
    use std::io::stdin;

    // Write a prompt to stdout and flush it.
    write!(stdout(), "$ ").map_err(CliError::WriteError)?;
    stdout().flush().map_err(CliError::WriteError)?;

    // Read a line from stdin and return it.
    let mut buffer = String::new();
    stdin().read_line(&mut buffer).map_err(CliError::ReadError)?;

    // Return the buffer
    Ok(buffer)
}

impl<OSCmd, ReplCmd, A> UiBuidableFiller for dyn CliUI<OneShotCommand = OSCmd, ReplCommand = ReplCmd, Args = A> {
    #[instrument]
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

    #[instrument]
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
