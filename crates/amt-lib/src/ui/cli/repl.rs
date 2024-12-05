/*
 * Copyright (C) 2023-2024. James Draycott me@racci.dev
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

use crate::ui::cli::error::CliError;
use crate::ui::cli::CliResult;
use clap::{CommandFactory, FromArgMatches};
use tracing::{debug, error, instrument};

crate::handler!(pub Repl<Option<O>> [
    #[arg(long, short = 'r', action = clap::ArgAction::SetTrue)]
    pub repl: bool
] {
    /// Run in REPL mode (Read-Eval-Print-Loop)
    ///
    /// This is a mode where the user can enter commands and have them executed
    /// in a loop until they exit.
    #[doc(hidden)]
    async fn repl(&mut self) -> CliResult<()>
    where
        Self: Sized,
    {
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

    #[doc(hidden)]
    #[instrument(level = "TRACE", err, ret, skip(self))]
    async fn respond(&mut self, line: &str) -> CliResult<bool> {
        let args = shlex::split(line).ok_or_else(|| CliError::ParseError(line.into()))?;
        let mut matches = Self::ReplAction::command().try_get_matches_from(&args).map_err(CliError::InvalidCommand)?;
        let parsed = Self::ReplAction::from_arg_matches_mut(&mut matches).map_err(CliError::InvalidCommand)?;

        <Self as ReplHandler>::handle(self, parsed, &Default::default()).await?;
        Ok(false)
    }
});

#[instrument(level = "TRACE", err, ret)]
fn readline() -> CliResult<String> {
    use std::io::{stdin, stdout, Write};

    stdout().write(b"$ ").map_err(CliError::WriteError)?;
    stdout().flush().map_err(CliError::WriteError)?;

    // Read a line from stdin and return it.
    let mut buffer = String::new();
    stdin().read_line(&mut buffer).map_err(CliError::ReadError)?;

    // Return the buffer
    Ok(buffer)
}
