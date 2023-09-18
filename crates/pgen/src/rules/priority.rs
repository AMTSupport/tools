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

use std::cmp::Ordering;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
#[repr(u8)]
pub enum Priority {
    High,
    Medium,
    Low,
    Custom(u8),
}

impl Priority {
    pub fn u8(&self) -> u8 {
        match self {
            Self::High => 90,
            Self::Medium => 50,
            Self::Low => 10,
            Self::Custom(value) => *value,
        }
    }
}
impl Ord for Priority {
    fn cmp(&self, other: &Self) -> Ordering {
        self.u8().cmp(&other.u8())
    }
}

impl PartialOrd for Priority {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.u8().partial_cmp(&other.u8())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn comparisons() {
        assert!(Priority::High > Priority::Medium);
        assert!(Priority::High > Priority::Low);
        assert!(Priority::Medium > Priority::Low);

        assert!(Priority::Low < Priority::Medium);
        assert!(Priority::Low < Priority::High);
        assert!(Priority::Medium < Priority::High);

        assert!(Priority::High >= Priority::Medium);
        assert!(Priority::High >= Priority::Low);
        assert!(Priority::Medium >= Priority::Low);

        assert!(Priority::Low <= Priority::Medium);
        assert!(Priority::Low <= Priority::High);
        assert!(Priority::Medium <= Priority::High);

        assert_eq!(Priority::High, Priority::High);
        assert_eq!(Priority::Medium, Priority::Medium);
        assert_eq!(Priority::Low, Priority::Low);

        assert_ne!(Priority::High, Priority::Medium);
        assert_ne!(Priority::High, Priority::Low);
        assert_ne!(Priority::Medium, Priority::Low);
    }
    
    #[test]
    fn order() {
        let mut vec = vec![Priority::Medium, Priority::High, Priority::Low];
        vec.sort();
        
        assert_eq!(vec, vec![Priority::Low, Priority::Medium, Priority::High]);
    }
    
    #[test]
    fn custom() {
        assert!(Priority::Custom(100) > Priority::High);
        assert!(Priority::Low > Priority::Custom(0));
        assert!(Priority::Custom(1) > Priority::Custom(0));
    }
}
