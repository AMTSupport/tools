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

use thiserror::Error;

#[derive(Debug, Error)]
pub enum CliError {
    #[error("Failed to receive line buffer: {0}")]
    BufError(#[source] std::io::Error),

    #[error("Failed to parse line buffer: {0}")]
    ParseError(String),

    #[error("Failed to write to stdout: {0}")]
    WriteError(#[source] std::io::Error),

    #[error("Failed to read from stdin: {0}")]
    ReadError(#[source] std::io::Error),

    #[error("Failed to parse command: {0}")]
    InvalidCommand(#[source] clap::Error),

    #[error("{0}")]
    Custom(String),
}
