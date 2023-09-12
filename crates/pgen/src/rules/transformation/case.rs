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
use crate::rules::priority::Priority;
use crate::rules::rule::Rule;
use clap::{Parser, ValueEnum};
use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Copy, Clone, Parser, Serialize, Deserialize, ValueEnum)]
pub enum CaseTransformation {
    None,
    #[default]
    Capitalise,
    AllExcludingFirst,
    Uppercase,
    Random,
    Alternating,
}

impl Rule for CaseTransformation {
    fn process_word(
        &self,
        _previous: Option<&Word>,
        _current: &Word,
        _last: bool,
        _passable: &mut Self::Passable,
    ) -> Vec<Action> {
        let copy = self.clone();
        vec![Action::Transformation(Priority::High, move |str| match &copy {
            CaseTransformation::None => str.to_string(),
            CaseTransformation::Uppercase => str.to_ascii_uppercase(),
            CaseTransformation::Capitalise => str.replacen(&str[0..1], &str[0..1].to_uppercase(), 1),
            CaseTransformation::AllExcludingFirst => {
                str.replacen(&str[1..str.len()], &str[1..str.len()].to_uppercase(), 1)
            }
            CaseTransformation::Alternating => str
                .chars()
                .enumerate()
                .map(|(i, c)| if i % 2 == 0 { c.to_ascii_uppercase() } else { c })
                .collect::<String>(),
            CaseTransformation::Random => {
                use rand::distributions::{Bernoulli, Distribution};

                let mut rng = rand::thread_rng();
                let bernoulli = Bernoulli::new(0.5).unwrap();
                str.chars()
                    .map(|c| match bernoulli.sample(&mut rng) {
                        true => c.to_ascii_uppercase(),
                        false => c,
                    })
                    .collect::<String>()
            }
        })]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::processor::processor::Processor;

    const STR: &str = "watermelon";

    #[test]
    fn none() {
        let mut processor = Processor::new(vec![STR]);
        CaseTransformation::None.process(&mut processor);

        assert_eq!(STR, processor.finish());
    }

    #[test]
    fn uppercase() {
        let mut processor = Processor::new(vec![STR]);
        CaseTransformation::Uppercase.process(&mut processor);

        assert_eq!("WATERMELON", processor.finish());
    }

    #[test]
    fn capitalise() {
        let mut processor = Processor::new(vec![STR]);
        CaseTransformation::Capitalise.process(&mut processor);

        assert_eq!("Watermelon", processor.finish());
    }

    #[test]
    fn all_excluding_first() {
        let mut processor = Processor::new(vec![STR]);
        CaseTransformation::AllExcludingFirst.process(&mut processor);

        assert_eq!("wATERMELON", processor.finish());
    }

    #[test]
    fn alternating() {
        let mut processor = Processor::new(vec![STR]);
        CaseTransformation::Alternating.process(&mut processor);

        assert_eq!("WaTeRmElOn", processor.finish());
    }

    #[test]
    fn random() {
        let mut processor = Processor::new(vec![STR]);
        CaseTransformation::Random.process(&mut processor);

        assert_eq!(STR.len(), processor.finish().len());
    }
}
