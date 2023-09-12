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

use crate::processor::word::Word;
use crate::rules::action::Action;
use crate::rules::position::Position;
use crate::rules::priority::Priority;
use crate::rules::rule::Rule;
use clap::{Args, ValueEnum};
use macros::EnumNames;
use rand::Rng;
use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, Args, Serialize, Deserialize)]
pub struct DigitAddition {
    /// The mode that is used to select where the digits are inserted
    #[arg(long = "digit-mode", default_value = "sandwhich-all")]
    pub fill_mode: FillMode,

    /// The mimimum number of digits to add to each filled area.
    ///
    /// # Example
    ///
    /// ```rust
    /// use regex::Regex;
    /// use rpgen::rules::addition::digits::{DigitAddition, FillMode};
    /// use rpgen::rules::rule::Rule;
    ///
    /// let rule = DigitAddition {
    ///     minimum: 1,
    ///     maximum: 6,
    ///     fill_mode: FillMode::SandwhichAll,
    /// };
    ///
    /// let mut  processor = rpgen::processor::processor::Processor::new(vec!["hello", "world"]);
    /// rule.process(&mut processor);
    ///
    /// // The result will be match the regex `r"\d{1,6}helloworld\d{1,6}"`
    /// assert!(Regex::new(r"\d{1,6}helloworld\d{1,6}").unwrap().is_match(&*processor.finish()));
    /// ```
    #[arg(long = "digit-minimum", default_value_t = 3, value_parser = clap::value_parser!(u8).range(1..=8))]
    pub minimum: u8,

    #[arg(long = "digit-maximum", default_value_t = 3, value_parser = clap::value_parser!(u8).range(1..=8))]
    pub maximum: u8,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize, ValueEnum, EnumNames)]
pub enum FillMode {
    /// Add digits before and after each word (this can get messy).
    ///
    /// Example:
    /// ```text
    /// [hello] -> "1hello1"
    /// [hello,world] -> "1hello11world1"
    /// ```
    SandwhichEach,

    /// Add digits before the first and last words.
    ///
    /// Example:
    /// ```text
    /// [hello] -> "1hello1"
    /// [hello,world] -> "1helloworld1"
    #[default]
    SandwhichAll,

    /// Add digits before each word.
    ///
    /// Example:
    /// ```text
    /// [hello] -> "1hello"
    /// [hello,world] -> "1hello1world"
    /// ```
    BeforeEach,

    /// Add digits before the first word.
    ///
    /// Example:
    /// ```text
    /// [hello] -> "1hello"
    /// [hello,world] -> "1helloworld"
    ///
    BeforeAll,

    /// Add digits after each word.
    ///
    /// Example:
    /// ```text
    /// [hello] -> "hello1"
    /// [hello,world] -> "hello1world1"
    /// ```
    AfterEach,

    /// Add digits after the last word.
    ///
    /// Example:
    /// ```text
    /// [hello] -> "hello1"
    /// [hello,world] -> "helloworld1"
    /// ```
    AfterAll,
}

impl FillMode {
    /// Get the positions to fill.
    ///
    /// The first bool is whether it is the first word.
    /// The second bool is whether it is the last word.
    ///
    /// The returned vector is the positions to fill.
    pub fn positions(&self, first: bool, last: bool) -> Vec<Position> {
        match self {
            Self::SandwhichEach => vec![Position::Start, Position::End],
            Self::SandwhichAll => match (first, last) {
                (true, true) => vec![Position::Start, Position::End],
                (true, false) => vec![Position::Start],
                (false, true) => vec![Position::End],
                (false, false) => vec![],
            },
            Self::BeforeEach => vec![Position::Start],
            Self::BeforeAll => match (first, last) {
                (true, true) | (true, false) => vec![Position::Start],
                _ => vec![],
            },
            Self::AfterEach => vec![Position::End],
            Self::AfterAll => match (first, last) {
                (true, true) | (false, true) => vec![Position::End],
                _ => vec![],
            },
        }
    }
}

impl Rule for DigitAddition {
    fn process_word(
        &self,
        previous: Option<&Word>,
        _current: &Word,
        last: bool,
        _passable: &mut Self::Passable,
    ) -> Vec<Action> {
        let positions = self.fill_mode.positions(previous.is_none(), last);
        if positions.is_empty() {
            return Vec::new();
        }

        let mut seed = rand::thread_rng();
        positions
            .iter()
            .map(|position| {
                let mut digits = String::new();
                while digits.len() < self.minimum as usize
                    || digits.len() < self.maximum as usize && seed.gen_bool(self.minimum as f64 / self.maximum as f64)
                {
                    let digit = seed.gen_range(0..9);
                    digits.push(char::from_digit(digit, 10).unwrap());
                }

                Action::Addition(Priority::High, *position, digits)
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::processor::processor::Processor;
    use regex::Regex;

    #[test]
    fn fill_sandwhich_each() {
        let mut processor = Processor::new(vec!["hello", "world"]);
        DigitAddition {
            minimum: 3,
            maximum: 3,
            fill_mode: FillMode::SandwhichEach,
        }
        .process(&mut processor);

        let result = processor.finish();
        assert!(Regex::new(r"\d{3,}hello\d{6,}world\d{3,}").unwrap().is_match(&result));
    }

    #[test]
    fn fill_sandwhich_all() {
        let mut processor = Processor::new(vec!["hello", "world"]);
        DigitAddition {
            minimum: 3,
            maximum: 3,
            fill_mode: FillMode::SandwhichAll,
        }
        .process(&mut processor);

        let result = processor.finish();
        assert!(Regex::new(r"\d{3,}helloworld\d{3,}").unwrap().is_match(&result));
    }

    #[test]
    fn fill_before_each() {
        let mut processor = Processor::new(vec!["hello", "world"]);
        DigitAddition {
            minimum: 3,
            maximum: 3,
            fill_mode: FillMode::BeforeEach,
        }
        .process(&mut processor);

        let result = processor.finish();
        assert!(Regex::new(r"\d{3,}hello\d{3,}world").unwrap().is_match(&result));
    }

    #[test]
    fn fill_before_all() {
        let mut processor = Processor::new(vec!["hello", "world"]);
        DigitAddition {
            minimum: 3,
            maximum: 3,
            fill_mode: FillMode::BeforeAll,
        }
        .process(&mut processor);

        let result = processor.finish();
        assert!(Regex::new(r"\d{3,}helloworld").unwrap().is_match(&result));
    }

    #[test]
    fn fill_after_each() {
        let mut processor = Processor::new(vec!["hello", "world"]);
        DigitAddition {
            minimum: 3,
            maximum: 3,
            fill_mode: FillMode::AfterEach,
        }
        .process(&mut processor);

        let result = processor.finish();
        assert!(Regex::new(r"hello\d{3,}world\d{3,}").unwrap().is_match(&result));
    }

    #[test]
    fn fill_after_all() {
        let mut processor = Processor::new(vec!["hello", "world"]);
        DigitAddition {
            minimum: 3,
            maximum: 3,
            fill_mode: FillMode::AfterAll,
        }
        .process(&mut processor);

        let result = processor.finish();
        assert!(Regex::new(r"helloworld\d{3,}").unwrap().is_match(&result));
    }
}
