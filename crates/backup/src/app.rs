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

use crate::config::runtime::Runtime;
use clap::Parser;
use lib::cli::Flags;
use lib::runtime::runtime::Cli;
use std::fmt::Debug;
use std::path::PathBuf;

#[derive(Debug)]
pub struct App {
    pub(crate) running: bool,
    pub(crate) runtime: Runtime,
}

#[derive(Debug, Clone, Parser)]
#[command(name = env!["CARGO_PKG_NAME"], version, author, about)]
pub struct AppCli {
    #[command(flatten)]
    pub flags: Flags,

    #[arg(short = 'D', long, help = "The path to the backup location root directory.")]
    pub destination: Option<PathBuf>,

    #[cfg(feature = "ui-cli")]
    #[clap(subcommand)]
    pub action: crate::ui::cli::action::Action,
}

impl Cli for AppCli {
    fn flags(&self) -> &Flags {
        &self.flags
    }
}

impl App {
    /// Constructs a new instance of [`App`].
    pub fn new(runtime: Runtime) -> Self {
        Self {
            running: false,
            runtime,
        }
    }
}

// impl App {
//     /// Handles the tick event of the terminal.
//     pub fn tick(&self) {}
//
//     /// Set running to false to quit the application.
//     pub fn quit(&mut self) {
//         self.running = false;
//     }
// }
