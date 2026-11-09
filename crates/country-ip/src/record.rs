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
    pub fn start(&self) -> IpAddr {
        match self {
            Record::RegistryRecord { value, .. } => *value,
            Record::DBRecord { start, .. } => *start,
        }
    }

    /// Inclusive bounds of the record, or [`None`] when the record is malformed.
    ///
    /// A registry record stores the IPv4 size as an address count and the IPv6 size as a prefix length,
    /// so both are resolved here and rejected if they overflow or fall outside the family.
    pub fn bounds(&self) -> Option<(IpAddr, IpAddr)> {
        let (start, end) = match self {
            Record::DBRecord { start, end, .. } => (*start, *end),
            Record::RegistryRecord { value, range, .. } => (*value, registry_end(*value, *range)?),
        };

        (start.is_ipv4() == end.is_ipv4() && start <= end).then_some((start, end))
    }

    pub fn end(&self) -> Option<IpAddr> {
        self.bounds().map(|(_, end)| end)
    }

    pub fn range(&self) -> Option<RangeInclusive<IpAddr>> {
        self.bounds().map(|(start, end)| start..=end)
    }

    pub fn u128_range(&self) -> Option<RangeInclusive<u128>> {
        let (start, end) = self.bounds()?;
        Some(as_u128(start)..=as_u128(end))
    }

    pub fn contains(&self, ip: &IpAddr) -> bool {
        let Some((start, end)) = self.bounds() else {
            return false;
        };

        // `IpAddr` orders every IPv4 address before every IPv6 address, so mixed families must
        // be rejected explicitly rather than relying on the bound comparison.
        start.is_ipv4() == ip.is_ipv4() && start <= *ip && *ip <= end
    }

    pub fn random(&self) -> Option<IpAddr> {
        use rand::prelude::{Rng, SeedableRng, SmallRng};

        let raw = SmallRng::from_entropy().gen_range(self.u128_range()?);
        Some(match self.start() {
            IpAddr::V4(_) => IpAddr::V4(Ipv4Addr::from(u32::try_from(raw).ok()?)),
            IpAddr::V6(_) => IpAddr::V6(Ipv6Addr::from(raw)),
        })
    }
}

/// Last address of a registry record, or [`None`] if its size or alignment is invalid.
fn registry_end(value: IpAddr, size: u32) -> Option<IpAddr> {
    match value {
        IpAddr::V4(start) => {
            let last = start.to_bits().checked_add(size.checked_sub(1)?)?;
            Some(IpAddr::V4(Ipv4Addr::from(last)))
        }
        IpAddr::V6(start) => {
            let prefix = u8::try_from(size).ok()?;
            Some(IpAddr::V6(Ipv6Inet::new(start, prefix).ok()?.last_address()))
        }
    }
}

fn as_u128(ip: IpAddr) -> u128 {
    match ip {
        IpAddr::V4(addr) => u128::from(u32::from(addr)),
        IpAddr::V6(addr) => u128::from(addr),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn registry(value: IpAddr, size: u32) -> Record {
        Record::RegistryRecord {
            registry: Registry::Apnic,
            alpha: Alpha2::AU,
            value,
            range: size,
            date: String::new(),
            status: Status::Allocated,
        }
    }

    #[test]
    fn ipv4_address_count_bounds() {
        let record = registry(IpAddr::V4(Ipv4Addr::new(101, 160, 0, 0)), 2);
        assert_eq!(
            record.bounds(),
            Some((
                IpAddr::V4(Ipv4Addr::new(101, 160, 0, 0)),
                IpAddr::V4(Ipv4Addr::new(101, 160, 0, 1))
            ))
        );
    }

    #[test]
    fn ipv4_address_count_overflow_is_rejected() {
        assert_eq!(
            registry(IpAddr::V4(Ipv4Addr::new(255, 255, 255, 0)), 512).bounds(),
            None
        );
    }

    #[test]
    fn ipv4_zero_count_is_rejected() {
        assert_eq!(registry(IpAddr::V4(Ipv4Addr::new(101, 160, 0, 0)), 0).bounds(), None);
    }

    #[test]
    fn ipv6_prefix_length_bounds() {
        let record = registry(IpAddr::V6(Ipv6Addr::new(0x2001, 0x200, 0, 0, 0, 0, 0, 0)), 35);
        assert_eq!(
            record.bounds(),
            Some((
                IpAddr::V6(Ipv6Addr::new(0x2001, 0x200, 0, 0, 0, 0, 0, 0)),
                IpAddr::V6(Ipv6Addr::new(
                    0x2001, 0x200, 0x1fff, 0xffff, 0xffff, 0xffff, 0xffff, 0xffff
                ))
            ))
        );
    }

    #[test]
    fn ipv6_prefix_length_over_u8_is_rejected() {
        assert_eq!(
            registry(IpAddr::V6(Ipv6Addr::new(0x2001, 0x200, 0, 0, 0, 0, 0, 0)), 256).bounds(),
            None
        );
    }

    #[test]
    fn ipv6_prefix_longer_than_address_is_rejected() {
        assert_eq!(
            registry(IpAddr::V6(Ipv6Addr::new(0x2001, 0x200, 0, 0, 0, 0, 0, 0)), 129).bounds(),
            None
        );
    }

    #[test]
    fn contains_rejects_the_other_family() {
        let record = registry(IpAddr::V4(Ipv4Addr::new(101, 160, 0, 0)), 2);
        assert!(record.contains(&IpAddr::V4(Ipv4Addr::new(101, 160, 0, 1))));
        assert!(!record.contains(&IpAddr::V6(Ipv6Addr::LOCALHOST)));
    }
}
