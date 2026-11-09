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

use crate::record::Record;
use keshvar::Alpha2;
use std::net::IpAddr;

/// Address ranges sorted by start, keyed per family so lookups only compare like addresses.
#[derive(Debug, Clone, Default)]
pub struct RangeIndex {
    v4: Vec<(u32, u32, Alpha2)>,
    v6: Vec<(u128, u128, Alpha2)>,
}

impl RangeIndex {
    pub fn new(records: &[Record]) -> Self {
        let mut v4 = Vec::new();
        let mut v6 = Vec::new();

        for record in records {
            let Some((start, end)) = record.bounds() else {
                continue;
            };

            match (start, end) {
                (IpAddr::V4(start), IpAddr::V4(end)) => {
                    v4.push((start.to_bits(), end.to_bits(), *record.alpha()));
                }
                (IpAddr::V6(start), IpAddr::V6(end)) => {
                    v6.push((start.to_bits(), end.to_bits(), *record.alpha()));
                }
                _ => {}
            }
        }

        v4.sort_unstable_by_key(|(start, _, _)| *start);
        v6.sort_unstable_by_key(|(start, _, _)| *start);

        Self { v4, v6 }
    }

    pub fn lookup(&self, ip: &IpAddr) -> Option<Alpha2> {
        match ip {
            IpAddr::V4(addr) => find(&self.v4, addr.to_bits()),
            IpAddr::V6(addr) => find(&self.v6, addr.to_bits()),
        }
    }
}

/// Finds the range with the largest start that still contains `addr`.
///
/// Ranges from RIR and DB-IP dumps are largely non-overlapping, so this is `O(log n)` in practice.
/// Overlapping ranges fall back to scanning backwards, which lets the most specific range win.
fn find<T: Ord + Copy>(ranges: &[(T, T, Alpha2)], addr: T) -> Option<Alpha2> {
    let upper = ranges.partition_point(|(start, _, _)| *start <= addr);
    ranges[..upper]
        .iter()
        .rev()
        .find(|(_, end, _)| *end >= addr)
        .map(|(_, _, alpha)| *alpha)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{Ipv4Addr, Ipv6Addr};

    fn record(start: IpAddr, end: IpAddr, alpha: Alpha2) -> Record {
        Record::DBRecord { alpha, start, end }
    }

    fn ipv4(octets: [u8; 4]) -> IpAddr {
        IpAddr::V4(Ipv4Addr::from(octets))
    }

    fn ipv6(groups: [u16; 8]) -> IpAddr {
        IpAddr::V6(Ipv6Addr::from(groups))
    }

    #[test]
    fn finds_ipv4_addresses() {
        let index = RangeIndex::new(&[
            record(ipv4([1, 0, 0, 0]), ipv4([1, 0, 0, 255]), Alpha2::US),
            record(ipv4([101, 160, 0, 0]), ipv4([101, 191, 255, 255]), Alpha2::AU),
        ]);

        assert_eq!(index.lookup(&ipv4([101, 160, 0, 1])), Some(Alpha2::AU));
        assert_eq!(index.lookup(&ipv4([1, 0, 0, 10])), Some(Alpha2::US));
        assert_eq!(index.lookup(&ipv4([192, 168, 1, 1])), None);
    }

    #[test]
    fn finds_ipv6_addresses() {
        let index = RangeIndex::new(&[record(
            ipv6([0x2001, 0x200, 0, 0, 0, 0, 0, 0]),
            ipv6([0x2001, 0x200, 0x1fff, 0xffff, 0xffff, 0xffff, 0xffff, 0xffff]),
            Alpha2::JP,
        )]);

        assert_eq!(index.lookup(&ipv6([0x2001, 0x200, 0, 0, 0, 0, 0, 1])), Some(Alpha2::JP));
        assert_eq!(index.lookup(&ipv6([0x2001, 0x200, 0x2000, 0, 0, 0, 0, 0])), None);
    }

    #[test]
    fn families_do_not_leak_into_each_other() {
        let index = RangeIndex::new(&[record(ipv4([0, 0, 0, 0]), ipv4([0, 0, 0, 255]), Alpha2::US)]);

        assert_eq!(index.lookup(&ipv6([0, 0, 0, 0, 0, 0, 0, 1])), None);
    }

    #[test]
    fn most_specific_overlapping_range_wins() {
        let index = RangeIndex::new(&[
            record(ipv4([10, 0, 0, 0]), ipv4([10, 255, 255, 255]), Alpha2::US),
            record(ipv4([10, 1, 0, 0]), ipv4([10, 1, 255, 255]), Alpha2::AU),
        ]);

        assert_eq!(index.lookup(&ipv4([10, 1, 0, 1])), Some(Alpha2::AU));
        assert_eq!(index.lookup(&ipv4([10, 0, 0, 1])), Some(Alpha2::US));
        assert_eq!(index.lookup(&ipv4([10, 2, 0, 1])), Some(Alpha2::US));
    }
}
