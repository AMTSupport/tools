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

use clap::{Parser, Subcommand};
use lib::cli::Flags;
use std::net::IpAddr;

#[derive(Parser, Debug)]
#[command(name = env!["CARGO_PKG_NAME"], version, author, about)]
pub struct Cli {
    /// The command to run
    #[command(subcommand)]
    pub action: CliAction,

    #[command(flatten)]
    pub flags: Flags,
}

#[derive(Subcommand, Debug)]
pub enum CliAction {
    #[cfg(feature = "gui")]
    GUI,

    /// Get a random IP address for a country.
    /// or a random country if no country is specified
    /// Countries must be specified as ISO 3166-1 alpha-2 or alpha-3 codes (e.g. US, USA, GB, GBR)
    Get {
        country: Option<String>,

        /// Use IpV6 addresses instead of IpV4
        #[arg(long, short = '6', action = clap::ArgAction::SetTrue)]
        ipv6: bool,
    },

    /// Lookup an IP address and get the country it belongs to.
    Lookup { addr: IpAddr },
}

impl lib::runtime::runtime::Cli for Cli {
    fn flags(&self) -> &Flags {
        &self.flags
    }
}
