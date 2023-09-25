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

use crate::cli::Flags as CommonFlags;
use crate::ui::cli::error::CliError;
use crate::ui::{UiBuidableFiller, UiBuildable};
use anyhow::Result;
use clap::{CommandFactory, Parser, Subcommand};
use inquire::Text;
use std::fmt::Debug;
use tracing::{error, instrument, trace};
#[cfg(feature = "ui-repl")]
use {
    clap::FromArgMatches,
    std::io::{stdin, stdout, Write},
    tracing::debug,
};

#[derive(Debug, Parser)]
struct MaybeRepl<O: Subcommand> {
    #[command(subcommand)]
    pub oneshot: Option<O>,

    #[cfg(feature = "ui-repl")]
    #[arg(long, short = 'r', action = clap::ArgAction::SetTrue)]
    pub repl: bool,

    #[command(flatten)]
    pub flags: CommonFlags,
}

pub type CliResult<T> = Result<T, CliError>;

pub trait AsyncCliUI: CliUI + Send + Sync {
    async fn handle_command(&mut self, command: Self::OneShotCommand, flags: &CommonFlags) -> CliResult<()>;

    #[cfg(feature = "ui-repl")]
    async fn handle_repl_command(&mut self, command: Self::ReplCommand, flags: &CommonFlags) -> CliResult<bool>;

    /// Run the CLI Application.
    ///
    /// This will be run by parsing the std::env::args() with [`MaybeRepl`]
    /// and then running the appropriate command.
    ///
    /// If the MaybeRepl has [`MaybeRepl::repl`] set to true,
    /// and there is also a [`MaybeRepl::oneshot`] command,
    /// then the oneshot command will be run once as a repl command.
    async fn run(&mut self) -> CliResult<()>
    where
        Self: Sized,
    {
        let command = MaybeRepl::<Self::OneShotCommand>::parse();
        let mut has_run = false;

        if let Some(action) = command.oneshot {
            self.handle_command(action, &command.flags).await?;
            has_run = true;
        }

        #[cfg(feature = "ui-repl")]
        if command.repl {
            self.repl(None).await?;
            has_run = true;
        }

        if !has_run {
            MaybeRepl::<Self::OneShotCommand>::command().print_help().map_err(CliError::WriteError)?;
        }

        Ok(())
    }

    /// Run in REPL mode (Read-Eval-Print-Loop)
    ///
    /// This is a mode where the user can enter commands and have them executed
    /// in a loop until they exit.
    #[cfg(feature = "ui-repl")]
    async fn repl(&mut self, init: Option<(Self::ReplCommand, CommonFlags)>) -> CliResult<()>
    where
        Self: Sized,
    {
        // Run the initial command if there is one.
        if let Some((action, flags)) = init {
            self.handle_repl_command(action, &flags).await?;
        }

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

    #[cfg(feature = "ui-repl")]
    async fn respond(&mut self, line: &str) -> CliResult<bool> {
        let args = shlex::split(line).ok_or_else(|| CliError::ParseError(line.into()))?;
        let mut matches = Self::ReplCommand::command().try_get_matches_from(&args).map_err(CliError::InvalidCommand)?;
        let parsed = Self::ReplCommand::from_arg_matches_mut(&mut matches).map_err(CliError::InvalidCommand)?;

        self.handle_repl_command(parsed, &CommonFlags::default()).await?;
        Ok(false)
    }
}

pub trait CliUI {
    /// The command that will be used in one-shot mode
    type OneShotCommand: Subcommand;

    /// The command that will be used to parse REPL commands
    #[cfg(feature = "ui-repl")]
    type ReplCommand: Parser + Subcommand;

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

#[cfg(feature = "ui-repl")]
#[instrument(level = "TRACE", err, ret)]
fn readline() -> CliResult<String> {
    stdout().write(b"$ ").map_err(CliError::WriteError)?;
    stdout().flush().map_err(CliError::WriteError)?;

    // Read a line from stdin and return it.
    let mut buffer = String::new();
    stdin().read_line(&mut buffer).map_err(CliError::ReadError)?;

    // Return the buffer
    Ok(buffer)
}

crate::dyn_impl! {
    impl UiBuidableFiller where {
        #[cfg(feature = "ui-repl")]
        for CliUI<OneShotCommand=OSCmd,ReplCommand=ReplCmd,Args=A> where {
            OSCmd: Subcommand + CommandFactory + Debug,
            ReplCmd: Subcommand + Parser + CommandFactory + Debug,
            A: Debug
        },
        #[cfg(not(feature = "ui-repl"))]
        for CliUI<OneShotCommand=OSCmd,Args=A> where {
            OSCmd: Subcommand + CommandFactory + Debug,
            A: Debug
        }
    } for {
        #[instrument(level = "TRACE", ret, err)]
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

        #[instrument(level = "TRACE", ret, err)]
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
}

#[macro_export]
macro_rules! dyn_impl {
    (impl $trait:ty where {
        $(
            $(#[$cfg:meta])*
            for $receiver:path where {
                $(
                    $where_ident:ident: $where_ty:tt $(+ $where_ty_add:tt)*
                ),*
            }
        ),*
    } for $impl_body:tt) => {
        $(
            $(#[$cfg])*
            impl<$($where_ident),*> $trait for dyn $receiver where
                $($where_ident: $where_ty $(+ $where_ty_add)*),*
            $impl_body
        )*
    }
}
