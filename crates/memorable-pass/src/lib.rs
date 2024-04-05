/*
 * Copyright (C) 2023-2024. James Draycott me@racci.dev
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, version 3.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.
 * See the GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program. If not, see https://www.gnu.org/licenses/.
 */

#![feature(lazy_cell)]
#![feature(assert_matches)]
#![feature(type_alias_impl_trait)]
#![feature(associated_type_defaults)]

use std::sync::Arc;
use crate::config::asset::WORDS;
use crate::processor::processor::Processor;
use crate::rules::rule::Rule;
use crate::rules::Rules;
use rand::Rng;
use tokio_stream::{iter, StreamExt};
use tracing::{instrument, trace};

pub mod config;
pub mod processor;
pub mod rules;
pub mod ui;

pub async fn generate(rules: &Rules) -> Vec<String> {
    let rules = Arc::new(rules);
    iter(0..=rules.amount)
        .then(|_| random_words(rules.word_count, rules.word_length_min, rules.word_length_max))
        .map(|words| {
            let mut processor = Processor::new(words);
            rules.addition_digits.process(&mut processor);
            rules.addition_separator.process(&mut processor);
            rules.transformation_case.process(&mut processor);

            processor.finish()
        })
        .collect()
        .await
}

// TODO :: Turn into stream
#[instrument(level = "TRACE")]
pub async fn random_words<'a>(word_count: u8, word_length_min: u8, word_length_max: u8) -> Vec<&'a str> {
    let range = word_length_min..=word_length_max;
    let mut words = Vec::with_capacity(word_count as usize);
    let seed = &mut rand::thread_rng();

    trace!("Finding {} words within range {:?}", word_count, range);
    while words.len() < word_count as usize {
        let length = seed.gen_range(range.clone());
        let possible_words = WORDS.get(&(length as usize)).unwrap();
        let word = possible_words.get(seed.gen_range(0..possible_words.len())).unwrap().as_str();

        words.push(word);
    }

    words
}
