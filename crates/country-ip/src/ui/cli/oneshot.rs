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
use std::net::IpAddr;

#[derive(Debug, Parser)]
pub enum OneshotAction {
    /// Get a random IP address for a country.
    ///
    /// or a random country if no country is specified
    /// Countries must be specified as ISO 3166-1 alpha-2 or alpha-3 codes (e.g. US, USA, GB, GBR)
    Get {
        country: Option<String>,

        /// Use IpV6 addresses instead of IpV4
        #[arg(long, short = '6', action = clap::ArgAction::SetTrue)]
        ipv6: bool,
    },

    /// Lookup an IP address and get the country it belongs to.
    Lookup {
        /// The IP address to lookup.
        ///
        /// This can be either an IPv4 or IPv6 address.
        addr: IpAddr,
    },
}
