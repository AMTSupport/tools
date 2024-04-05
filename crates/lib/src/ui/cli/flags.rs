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


use clap::{Parser, ValueEnum};

/// A struct that contains common flags for the CLI
///
/// These flags are always available to the CLI,
/// though the implementation is up to the final application.
#[derive(Default, Debug, Clone, Copy, Parser)]
pub struct CommonFlags<const HIDE: bool = false> {
    /// The verbosity of the terminal logger
    #[arg(short, long, hide = HIDE, global = true, action = clap::ArgAction::Count)]
    pub verbose: u8,

    /// If there shouldn't be any changes made and only a dry run should be performed.
    #[arg(short, long, hide = HIDE, global = true, action = clap::ArgAction::SetTrue)]
    pub dry_run: bool,

    /// If the program should be run in a quiet mode
    #[arg(short, long, hide = HIDE, global = true, action = clap::ArgAction::SetTrue, default_value_if("format", "OutputFormat::Json", "true"))]
    pub quiet: bool,

    /// If the program should print in a JSON format
    #[cfg(feature = "ui-cli-formatting")]
    #[arg(short, long, hide = HIDE, global = true, value_enum, default_value_t = OutputFormat::Human)]
    pub format: OutputFormat,

    #[cfg(feature = "updater")]
    #[arg(short, long, hide = HIDE, global = true, action = clap::ArgAction::SetTrue)]
    pub update: bool,
}

/// The output format for the CLI
///
/// This is only available if the `ui-cli-formatting` feature is enabled.
/// When formatting is set to non `Human` the quiet mode / flag is implied.
#[cfg(feature = "ui-cli-formatting")]
#[derive(Default, Debug, Clone, Copy, ValueEnum)]
pub enum OutputFormat {
    /// Print the output in a human-readable format
    #[default]
    Human,

    /// Print the output in a JSON format
    Json,
}