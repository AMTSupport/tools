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
use crate::rules::action::{Action, ActionCondition};
use crate::rules::position::Position;
use crate::rules::priority::Priority;
use crate::rules::rule::Rule;
use clap::{Args, ValueEnum};
use macros::EnumNames;
use serde::{Deserialize, Serialize};

const POSSIBLE_CHARS: [char; 12] = ['!', '@', '$', '%', '.', '&', '*', '-', '+', '=', '?', ':'];

#[derive(Debug, Clone, Hash, Ord, PartialOrd, Eq, PartialEq, Args, Serialize, Deserialize)]
pub struct SeparatorAddition {
    #[arg(long = "separator-mode", default_value = "single")]
    pub(crate) mode: SeparatorMode,

    #[arg(long = "separator-chars", default_value_t = POSSIBLE_CHARS.iter().collect::<String>())]
    pub(crate) chars: String,
}

#[derive(
    Default, Debug, Clone, Copy, Hash, Ord, PartialOrd, Eq, PartialEq, Serialize, Deserialize, ValueEnum, EnumNames,
)]
pub enum SeparatorMode {
    None,
    #[default]
    Single,
    Random,
}

impl Default for SeparatorAddition {
    fn default() -> Self {
        Self {
            mode: SeparatorMode::Single,
            chars: POSSIBLE_CHARS.iter().collect::<String>(),
        }
    }
}

impl Rule for SeparatorAddition {
    type Passable = Option<char>;

    fn process_word(
        &self,
        _previous: Option<&Word>,
        _current: &Word,
        last: bool,
        passable: &mut Self::Passable,
    ) -> Vec<Action> {
        if self.chars.is_empty() || self.mode == SeparatorMode::None {
            return vec![];
        }

        let fn_random = || {
            use rand::distributions::{Distribution, Uniform};

            let mut rng = rand::thread_rng();
            let uniform = Uniform::new(0, self.chars.len());

            self.chars.chars().collect::<Vec<char>>()[uniform.sample(&mut rng)]
        };

        let mut fn_char = || match self.mode {
            SeparatorMode::None => '\0',
            SeparatorMode::Single => *passable.get_or_insert_with(fn_random),
            SeparatorMode::Random => fn_random(),
        };

        match last {
            // Last word is responsible for adding the end separator.
            true => vec![Position::Start, Position::End],
            // All other words are responsible for adding the start separator for themselves.
            false => vec![Position::Start],
        }
        .into_iter()
        .map(|position| {
            Action::Addition(
                position.positional_value(Priority::Low, Priority::Custom(100)),
                position,
                fn_char().to_string(),
                ActionCondition::HasInput,
            )
        })
        .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::processor::processor::Processor;
    use regex::Regex;
    use std::assert_matches::assert_matches;

    #[test_log::test(test)]
    fn separator_none() {
        let mut processor = Processor::new(vec!["hello", "world"]);
        SeparatorAddition {
            mode: SeparatorMode::None,
            ..SeparatorAddition::default()
        }
        .process(&mut processor);

        let result = processor.finish();
        assert_eq!("helloworld", result);
    }

    #[test]
    fn single() {
        let mut processor = Processor::new(vec!["hello", "world"]);
        SeparatorAddition {
            mode: SeparatorMode::Single,
            ..SeparatorAddition::default()
        }
        .process_with_passable(&mut processor, &mut Some('?'));

        let result = processor.finish();
        assert_eq!("hello?world?", result);
    }

    #[test]
    fn random() {
        let mut processor = Processor::new(vec!["hello", "world"]);
        SeparatorAddition {
            mode: SeparatorMode::Random,
            ..SeparatorAddition::default()
        }
        .process_with_passable(&mut processor, &mut None);

        let result = processor.finish();
        let group = "[!@$%\\.&*\\-+=?:]";
        println!("{}", result);
        assert_matches!(
            Regex::new(&format!("^hello{group}world{group}$")).unwrap().is_match(&result),
            true
        );
    }
}
