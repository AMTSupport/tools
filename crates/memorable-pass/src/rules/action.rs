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

#[derive(Derivative)]
#[derivative(Debug, Ord = "feature_allow_slow_enum", Eq, PartialEq)]
pub enum Action {
    Addition(Priority, Position, String),
    Transformation(
        Priority,
        #[derivative(Debug = "ignore", Ord = "ignore", PartialEq = "ignore")] TransformationFn,
    ),
}

impl PartialOrd for Action {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match (self, other) {
            (Self::Addition(pri1, pos1, _), Self::Addition(pri2, pos2, _)) => match pos1 == pos2 {
                true => pri1.partial_cmp(pri2),
                false => pos1.partial_cmp(pos2),
            },
            (Self::Transformation(pri1, _), Self::Transformation(pri2, _)) => pri1.partial_cmp(pri2),
            (Self::Addition(_, _, _), Self::Transformation(_, _)) => Some(Ordering::Less),
            (Self::Transformation(_, _), Self::Addition(_, _, _)) => Some(Ordering::Greater),
        }
    }
}
