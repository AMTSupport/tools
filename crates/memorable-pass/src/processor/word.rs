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

use crate::rules::action::Action;
use std::collections::BTreeSet;
use std::fmt::Display;

#[derive(Debug)]
pub struct Word<'a> {
    pub index: usize,
    pub word: &'a str,
    pub actions: BTreeSet<Action>,
}

impl<'a> Word<'a> {
    pub fn new(index: usize, word: &'a str) -> Self {
        Self {
            index,
            word,
            actions: BTreeSet::new(),
        }
    }
}

impl Display for Word<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}@{} w/ {:#?}", self.index, self.word, self.actions)
    }
}
