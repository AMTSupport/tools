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

use crate::cleaners::cleaner::Cleaner;
use clap::Parser;
use lib::ui::cli::flags::CommonFlags;

#[derive(Default, Debug, Parser)]
#[command(name = env!["CARGO_PKG_NAME"], version, author, about)]
pub struct Cli {
    #[command(flatten)]
    pub flags: CommonFlags,

    // #[command(subcommand)]
    // pub complete: Option<Shell>,

    #[arg(
    ignore_case = true,
    default_values_t = Cleaner::get_variants(),
    last = true
    )]
    pub cleaners: Vec<Cleaner>,
}
