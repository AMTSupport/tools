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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Position {
    Start,
    End,
}

impl Position {
    pub fn is_start(&self) -> bool {
        matches!(self, Self::Start)
    }

    pub fn is_end(&self) -> bool {
        matches!(self, Self::End)
    }

    pub fn positional_value<V>(&self, if_start: V, if_end: V) -> V {
        match self {
            Self::Start => if_start,
            Self::End => if_end,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn position_functions() {
        assert!(Position::Start.is_start());
        assert!(Position::End.is_end());
    }

    #[test]
    fn position_ordering() {
        assert!(Position::Start < Position::End);
        assert!(Position::End > Position::Start);
        assert!(Position::Start <= Position::End);
        assert!(Position::End >= Position::Start);

        assert_ne!(Position::Start, Position::End);
        assert_ne!(Position::End, Position::Start);

        assert_eq!(Position::Start, Position::Start);
        assert_eq!(Position::End, Position::End);
    }
}
