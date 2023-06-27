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

#![feature(lazy_cell)]
use crate::config::rules::Rules;
use clap::Subcommand;

pub mod config;
pub mod generation;

#[cfg(windows)]
const CONF_PATH: &str = "%APPDATA%\\.config\\pgen\\rules.toml";
#[cfg(unix)]
const CONF_PATH: &str = "~/.config/pgen/rules.toml";

#[derive(Subcommand)]
pub enum Commands {
    Generate {
        #[command(flatten)]
        rules: Rules,

        /// The file to use as the rules config.
        #[arg(short, long, default_value_t = CONF_PATH.into())]
        file: String,

        #[command(flatten)]
        flags: lib::cli::Flags,
    },
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },
}

#[derive(Subcommand)]
pub enum ConfigAction {
    // Validate {
    //     #[arg(short, long, default_value_t = CONF_PATH)]
    //     file: PathBuf,
    //
    //     #[command(flatten)]
    //     flags: lib::cli::Flags,
    // },
    Generate {
        #[arg(short, long, default_value_t = CONF_PATH.into())]
        file: String,

        #[command(flatten)]
        rules: Rules,

        /// Whether to overwrite the file if it already exists.
        #[arg(short, long)]
        force: bool,

        #[command(flatten)]
        flags: lib::cli::Flags,
    },
    Show {
        #[arg(short, long, default_value_t = CONF_PATH.into())]
        file: String,

        #[command(flatten)]
        flags: lib::cli::Flags,
    },
}
