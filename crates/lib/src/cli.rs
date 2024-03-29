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

use clap::Parser;

#[derive(Default, Debug, Clone, Copy, Parser)]
pub struct Flags<const HIDE: bool = false> {
    /// The verbosity of the terminal logger
    #[arg(short, long, hide = HIDE, global = true, action = clap::ArgAction::Count)]
    pub verbose: u8,

    /// If there shouldn't be any changes made and only a dry run should be performed
    #[arg(short, long, hide = HIDE, global = true, action = clap::ArgAction::SetTrue)]
    pub dry_run: bool,

    /// If the program should be run in a quiet mode
    #[arg(short, long, hide = HIDE, global = true, action = clap::ArgAction::SetTrue)]
    pub quiet: bool,
}
