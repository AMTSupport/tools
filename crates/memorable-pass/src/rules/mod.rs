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

pub mod action;
pub mod addition;
pub mod position;
pub mod priority;
pub mod rule;
pub mod transformation;

use crate::rules::addition::digits::DigitAddition;
use crate::rules::addition::separator::SeparatorAddition;
use crate::rules::transformation::case::CaseTransformation;
use clap::Parser;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

/// The rules which are used to generate passwords.
#[derive(Debug, Clone, Parser, Serialize, Deserialize)]
#[serde(default)]
pub struct Rules {
    /// How many words are used.
    ///
    /// This is the number of words which each password will contain.
    /// These words are the base of the password with the rules being applied to them.
    ///
    /// The maximum number of words is 10 with the minimum being 1.
    #[arg(short = 'w', long = "words", default_value_t = 2, value_parser = clap::value_parser!(u8).range(1..10))]
    pub word_count: u8,

    /// The minimum length of each word.
    #[arg(short = 'm', long = "min-length", default_value_t = 5, value_parser = clap::value_parser!(u8).range(3..=9))]
    pub word_length_min: u8,

    /// The maximum length of each word.
    #[arg(short = 'M', long = "max-length", default_value_t = 7, value_parser = clap::value_parser!(u8).range(3..=9))]
    pub word_length_max: u8,

    #[command(flatten)]
    pub addition_digits: DigitAddition,

    #[command(flatten)]
    pub addition_separator: SeparatorAddition,

    #[arg(long, default_value_t = CaseTransformation::default(), value_enum)]
    pub transformation_case: CaseTransformation,

    /// The number of passwords to generate.
    #[arg(short, long, default_value_t = 3)]
    pub amount: usize,
}

impl Default for Rules {
    fn default() -> Self {
        Rules {
            word_count: 2,
            word_length_min: 5,
            word_length_max: 7,
            addition_digits: DigitAddition::default(),
            addition_separator: SeparatorAddition::default(),
            transformation_case: CaseTransformation::default(),
            amount: 3,
        }
    }
}
