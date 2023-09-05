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

use crate::cleaners::cleaner::Cleaner;
use clap::Parser;
use clap_complete::dynamic::shells::CompleteCommand;
use lib::cli::Flags;

#[derive(Parser, Debug)]
#[command(name = env!["CARGO_PKG_NAME"], version, author, about)]
pub struct Cli {
    #[command(flatten)]
    pub flags: Flags,

    #[command(subcommand)]
    pub complete: Option<CompleteCommand>,

    #[arg(
        ignore_case = true,
        default_values_t = Cleaner::get_variants(),
        last = true
    )]
    pub cleaners: Vec<Cleaner>,
}

impl Default for Cli {
    fn default() -> Self {
        Self {
            cleaners: Cleaner::get_variants(),
            flags: Flags::default(),
            complete: None,
        }
    }
}
