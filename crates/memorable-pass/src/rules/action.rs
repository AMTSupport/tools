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

use crate::rules::position::Position;
use crate::rules::priority::Priority;
use crate::TransformationFn;
use derivative::Derivative;
use std::cmp::Ordering;
use tracing::instrument;

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq)]
pub enum ActionCondition {
    Always,
    HasInput,
    HasNoInput,
}

#[derive(Derivative)]
#[derivative(Debug, Eq)]
pub enum Action {
    Addition(Priority, Position, String, ActionCondition),
    Transformation(
        Priority,
        ActionCondition,
        #[derivative(Debug = "ignore")] TransformationFn,
    ),
}

impl ActionCondition {
    #[instrument(level = "TRACE", skip(self), ret)]
    pub fn should_use(&self, word_index: usize, position: &Position, word_range: &(usize, usize)) -> bool {
        match self {
            Self::Always => true,
            Self::HasInput => match position {
                Position::Start => word_index > 0 || word_range.0 > 0,
                Position::End => word_index > 0 || word_range.1 > 0,
            },
            Self::HasNoInput => match position {
                Position::Start => word_index == 0 && word_range.0 == 0,
                Position::End => word_index == 0 && word_range.0 == word_range.1,
            },
        }
    }
}

impl PartialEq<Self> for Action {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}

impl PartialOrd for Action {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Action {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (Self::Addition(pri1, pos1, ..), Self::Addition(pri2, pos2, ..)) => match pos1 == pos2 {
                true => pri1.cmp(pri2),
                false => pos1.cmp(pos2),
            },
            (Self::Transformation(pri1, ..), Self::Transformation(pri2, ..)) => pri1.cmp(pri2),
            (Self::Addition(..), Self::Transformation(..)) => Ordering::Less,
            (Self::Transformation(..), Self::Addition(..)) => Ordering::Greater,
        }
    }
}
