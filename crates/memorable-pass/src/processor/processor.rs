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
use tracing::debug;

pub struct Processor<'a> {
    pub(crate) words: Vec<Word<'a>>,
}

impl<'a> Processor<'a> {
    /// Create a new processor for the given words.
    pub fn new(raw: Vec<&'a str>) -> Self {
        let words = raw.iter().enumerate().map(|(index, word)| Word::new(index, word)).collect::<Vec<Word>>();
        Self { words }
    }

    /// Create the final result of the processor.
    ///
    /// This will apply all the rules to the words and return the final result.
    pub fn finish(&mut self) -> String {
        let mut result = String::new();

        for (index, word) in self.words.iter().enumerate() {
            debug!("Word: {word:#?}");

            let mut mut_word = word.word.to_string();
            let mut start_end = (0usize, mut_word.len());
            for action in word.actions.iter().rev() {
                debug!("Applying rule: {action:?} to range {start_end:?}");

                match action {
                    Action::Addition(_, position, addition, condition) => {
                        match condition.should_use(index, position, &start_end) {
                            false => debug!("Skipping rule as condition failed"),
                            // ActionCondition::HasNoInput if start_end.0 != 0 || position.is_end() => {
                            //     debug!("Skipping rule as it has no input and is not at the start of the word");
                            //     continue
                            // },
                            // ActionCondition::HasInput if start_end.0 == 0 && position.is_start() => {
                            //     debug!("Skipping rule as it has input and is at the start of the word");
                            //     continue
                            // },
                            _ => match position {
                                Position::Start => {
                                    mut_word.insert_str(start_end.0, addition);
                                    start_end = (start_end.0 + addition.len(), start_end.1 + addition.len())
                                }
                                Position::End => {
                                    mut_word.insert_str(start_end.1, addition);
                                    start_end = (start_end.0, start_end.1 + addition.len())
                                }
                            },
                        }
                    }
                    Action::Transformation(_, _condition, transformation) => {
                        let before = mut_word.len();
                        let ranged = &mut_word[start_end.0..start_end.1];
                        mut_word.replace_range(start_end.0..start_end.1, &transformation(ranged));
                        start_end = (
                            start_end.0 + (mut_word.len() - before),
                            start_end.1 + (mut_word.len() - before),
                        );
                    }
                }

                debug!("Applied rule: {action:?}; result: {mut_word}, range {start_end:?}");
            }

            result.push_str(&mut_word);
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rules::addition::digits::{DigitAddition, FillMode};
    use crate::rules::addition::separator::{SeparatorAddition, SeparatorMode};
    use crate::rules::rule::Rule;
    use crate::rules::transformation::case::CaseTransformation;
    use regex::Regex;

    #[test_log::test(test)]
    fn addition_start() {
        let mut processor = Processor::new(vec!["hello", "world"]);
        DigitAddition {
            minimum: 3,
            maximum: 3,
            fill_mode: FillMode::SandwichAll,
        }
        .process(&mut processor);

        let result = processor.finish();
        assert!(Regex::new("[0-9]{3,}helloworld").unwrap().is_match(&result));
    }

    #[test_log::test(test)]
    fn all_rules_processor() {
        let mut processor = Processor::new(vec!["hello", "world"]);
        SeparatorAddition {
            mode: SeparatorMode::Single,
            chars: "-".into(),
        }
        .process(&mut processor);

        DigitAddition {
            minimum: 3,
            maximum: 3,
            fill_mode: FillMode::SandwichAll,
        }
        .process(&mut processor);

        CaseTransformation::Uppercase.process(&mut processor);

        let result = processor.finish();
        assert!(Regex::new("[0-9]{3,}-HELLO-WORLD-[0-9]{3,}").unwrap().is_match(&result));
    }
}
