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

use crate::registry::Registry;
use cidr::Ipv6Inet;
use keshvar::Alpha2;
use macros::CommonFields;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use std::ops::RangeInclusive;
use tracing::instrument;

#[derive(Debug, Clone, CommonFields)]
pub enum Record {
    RegistryRecord {
        registry: Registry,
        alpha: Alpha2,
        value: IpAddr,
        range: u32,
        date: String,
        status: Status,
    },
    DBRecord {
        alpha: Alpha2,
        start: IpAddr,
        end: IpAddr,
    },
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Status {
    Assigned,
    Allocated,
    Reserved,
}

impl Record {
    #[instrument(level = "TRACE", ret)]
    pub fn start(&self) -> IpAddr {
        match self {
            Record::RegistryRecord { value, .. } => *value,
            Record::DBRecord { start, .. } => *start,
        }
    }

    #[instrument(level = "TRACE", ret)]
    pub fn end(&self) -> IpAddr {
        match self {
            Record::DBRecord { end, .. } => *end,
            Record::RegistryRecord { value, range, .. } => match value {
                IpAddr::V4(start) => {
                    let bits = start.to_bits();
                    let end = bits + (*range - 1);
                    IpAddr::V4(Ipv4Addr::from(end))
                }
                IpAddr::V6(start) => {
                    let cidr = Ipv6Inet::new(*start, *range as u8).unwrap();
                    IpAddr::V6(cidr.last_address())
                }
            },
        }
    }

    #[instrument(level = "TRACE", ret)]
    pub fn range(&self) -> RangeInclusive<IpAddr> {
        self.start()..=self.end()
    }

    #[instrument(level = "TRACE", ret)]
    pub fn u128_range(&self) -> RangeInclusive<u128> {
        let start = match self.start() {
            IpAddr::V4(addr) => addr.to_bits() as u128,
            IpAddr::V6(addr) => addr.to_bits(),
        };
        let end = match self.end() {
            IpAddr::V4(addr) => addr.to_bits() as u128,
            IpAddr::V6(addr) => addr.to_bits(),
        };
        start..=end
    }

    #[instrument(level = "TRACE", ret)]
    pub fn contains(&self, ip: &IpAddr) -> bool {
        self.range().contains(ip)
    }

    #[instrument(level = "TRACE", ret)]
    pub fn random(&self) -> IpAddr {
        use rand::prelude::{Rng, SeedableRng, SmallRng};

        let mut rng = SmallRng::from_entropy();
        let raw = rng.gen_range(self.u128_range());
        match self.start() {
            IpAddr::V4(_) => IpAddr::V4(Ipv4Addr::from(raw as u32)),
            IpAddr::V6(_) => IpAddr::V6(Ipv6Addr::from(raw)),
        }
    }
}
