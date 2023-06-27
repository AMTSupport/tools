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

use serde::{Deserialize, Serialize};

#[derive(
    Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, clap::ValueEnum
)]
pub enum SeparatorMode {
    None,
    Random,
}

pub(crate) const CHARS: [char; 12] = ['!', '@', '$', '%', '.', '&', '*', '-', '+', '=', '?', ':'];

impl SeparatorMode {
    pub fn get(&self, chars: &[char; 12]) -> char {
        match self {
            SeparatorMode::None => '\0',
            SeparatorMode::Random => {
                use rand::distributions::{Distribution, Uniform};

                let mut rng = rand::thread_rng();
                let uniform = Uniform::new(0, CHARS.len());

                chars.get(uniform.sample(&mut rng)).unwrap().clone()
            }
        }
    }
}
