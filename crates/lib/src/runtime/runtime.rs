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

use crate::cli::Flags;
use anyhow::{Error, Result};
use clap::Parser;
use std::sync::RwLock;
use tracing::dispatcher::DefaultGuard;
use tracing::error;
use tracing::metadata::LevelFilter;
use tracing_subscriber::fmt::format::FmtSpan;
use tracing_subscriber::util::SubscriberInitExt;

pub trait Cli: Parser + Send + Sync + 'static {
    fn flags(&self) -> &Flags;
}

pub trait Runtime<C: Cli>: Send + Sync + 'static {
    fn new() -> Result<Self>
    where
        Self: Sized;

    fn new_logger(flags: &Flags) -> DefaultGuard {
        let (level, span) = match flags.verbose {
            0 => (LevelFilter::INFO, FmtSpan::NONE),
            1 => (LevelFilter::DEBUG, FmtSpan::NONE),
            2 => (LevelFilter::TRACE, FmtSpan::NONE),
            _ => (LevelFilter::TRACE, FmtSpan::FULL),
        };

        let builder = tracing_subscriber::fmt()
            .without_time()
            .with_thread_names(flags.verbose > 2)
            .with_thread_ids(flags.verbose > 2)
            .with_level(flags.verbose > 0)
            .with_line_number(flags.verbose > 0)
            .with_max_level(level)
            .with_span_events(span)
            .with_target(false);

        builder.finish().set_default()
    }

    fn new_cli() -> Result<C> {
        Ok(C::parse())
    }

    fn new_errors() -> RwLock<Vec<Error>> {
        RwLock::new(Vec::new())
    }

    fn sumbit_error(&mut self, error: Error) -> Result<()> {
        // If verbosity is greater than 0, print the error at runtime.
        if self.__get_cli().flags().verbose > 0 {
            error!("Error: {}", error);
        }

        let errors = self.__get_errors();
        let mut write = errors.write().unwrap();
        write.push(error);
        drop(write);

        Ok(())
    }

    fn __get_cli(&self) -> &C;
    fn __get_errors(&mut self) -> &mut RwLock<Vec<Error>>;
}
