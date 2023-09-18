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

use anyhow::Result;
use rand::{rngs::StdRng, Rng, RngCore, SeedableRng};
use tracing::{debug, trace};

pub struct Generator {
    pub rules: Rules,
    seed: StdRng,
}

pub struct Insertion {
    pub index: usize,
    pub value: String,
}

pub trait GeneratorFunctions: Sized {
    fn new(rules: Rules) -> Result<Self>;

    async fn generate<'r>(&mut self) -> Result<Vec<String>>;

    async fn get_words<R: RngCore>(
        seed: &mut R,
        word_count: &usize,
        word_length_min: &usize,
        word_length_max: &usize,
        transformation: &Transformation,
    ) -> Vec<String>;

    async fn get_digits(seed: &mut StdRng, amount: &usize) -> String;

    async fn get_chars(&self, amount: &usize) -> Vec<char>;
}

impl GeneratorFunctions for Generator {
    fn new(rules: Rules) -> Result<Self> {
        trace!("Initialising generator state");
        Ok(Generator {
            rules,
            seed: StdRng::from_entropy(),
        })
    }

    async fn generate<'r>(&mut self) -> Result<Vec<String>> {
        fn separator_count(word_count: &usize, mode: &SeparatorMode) -> usize {
            match mode {
                SeparatorMode::None => 0,
                SeparatorMode::Random => word_count - 1,
            }
        }

        let max_length = (&self.rules.word_length_max * &self.rules.word_count)
            + (&self.rules.digits_before + &self.rules.digits_after)
            + (separator_count(&self.rules.word_count, &self.rules.separator_mode));

        debug!(
            "Generating {count} passwords with a max length of {max_length}",
            count = &self.rules.amount
        );

        let rules = self.rules.clone();
        let seed = self.seed.clone();
        let len = self.rules.amount.clone();
        let mut passwords = Vec::with_capacity((&self.rules.amount).clone());
        while passwords.len() < len {
            // let password = Arc::new(tokio::sync::RwLock::new(String::with_capacity(
            //     max_length.clone(),
            // )));
            let mut seed = seed.clone();
            let mut handles = Vec::with_capacity(4);

            handles.push(tokio::spawn(async move {
                let words = Generator::get_words(
                    &mut seed,
                    &rules.word_count,
                    &rules.word_length_min,
                    &rules.word_length_max,
                    &rules.transform,
                )
                .await;

                let mut for_separator = Vec::with_capacity(separator_count(&rules.word_count, &rules.separator_mode));
                let mut last_index = 0;
                let mut insertions = vec![];
                for (index, word) in words.iter().enumerate() {
                    let inserted_at = match index {
                        0 => match &rules.digits_before {
                            0 => 0,
                            digits => digits + 1,
                        },
                        _ => {
                            for_separator.push(&last_index + 1);
                            match &rules.separator_mode {
                                SeparatorMode::None => &last_index + 1,
                                SeparatorMode::Random => &last_index + 2,
                            }
                        }
                    };

                    // let mut lock = password.blocking_write();

                    insertions.push(Insertion {
                        index: inserted_at,
                        value: word.to_string(),
                    });
                    // lock.insert_str(inserted_at, &word);
                    last_index = &inserted_at + word.len();

                    // drop(lock);
                }

                (insertions, Some(for_separator))
            }));

            // if self.rules.digits_before > 0 {
            //     handles.push(tokio::spawn(async {
            //         let digits =
            //             Generator::get_digits(&mut self.seed, &self.rules.digits_before).await;
            //
            //         let mut lock = password.blocking_write();
            //         lock.insert_str(0, &digits);
            //         drop(lock);
            //
            //         None
            //     }));
            // }
            //
            // if self.rules.digits_after > 0 {
            //     handles.push(tokio::spawn(async {
            //         let digits =
            //             Generator::get_digits(&mut self.seed, &self.rules.digits_after).await;
            //
            //         let mut lock = password.blocking_write();
            //
            //         let inserted_at = &max_length - digits.len();
            //         lock.insert_str(inserted_at, &digits);
            //
            //         drop(lock);
            //
            //         None
            //     }))
            // }
            //
            // if self.rules.separator_mode != SeparatorMode::None {
            //     handles.push(tokio::spawn(async {
            //         let mut chars = self.get_chars(&separator_count).await.iter_mut();
            //         let mut indexes = handles[0]
            //             .await
            //             .context("Retrieve indexes for needed separators")
            //             .unwrap()
            //             .unwrap();
            //
            //         let mut lock = password.blocking_write();
            //
            //         while let Some(index) = indexes.pop() {
            //             lock.insert(index, chars.next().unwrap().clone());
            //         }
            //
            //         drop(lock);
            //
            //         None
            //     }))
            // }

            passwords.push(tokio::spawn(async {
                let password = String::new();
                for handle in handles {
                    let (insertions, separators) = handle.await.unwrap();
                    let mut password = password.clone();
                    for insertion in insertions {
                        password.insert_str(insertion.index, &insertion.value);
                    }
                }

                password
            }))
        }

        let mut outputs = Vec::with_capacity(passwords.len());
        for password_handle in passwords {
            trace!("Awaiting password");
            outputs.push(password_handle.await?);
        }

        Ok(outputs)
    }

    async fn get_words<R: RngCore>(
        seed: &mut R,
        word_count: &usize,
        word_length_min: &usize,
        word_length_max: &usize,
        transformation: &Transformation,
    ) -> Vec<String> {
        let mut words = Vec::with_capacity(word_count.clone());

        while words.len() < *word_count {
            let range = *word_length_min..*word_length_max;
            trace!("Getting length of word from range {range:#?}.");
            let length = seed.gen_range(range);
            trace!("Looking for word with length {length}.");

            let possible_words = (&WORDS).get(&length).unwrap();
            trace!(
                "Found {count} words with length {length}.",
                count = possible_words.len()
            );

            let word = possible_words.get(seed.gen_range(0..possible_words.len())).unwrap().clone();
            trace!("Selected word: {word}");

            let word = transformation.transform(word);
            trace!("Transformed word: {word}");

            words.push(word);
        }

        words
    }

    async fn get_digits(seed: &mut StdRng, amount: &usize) -> String {
        trace!("Generating {amount} digits");
        let mut digits = String::with_capacity(amount.clone());

        while digits.len() < *amount {
            let digit = seed.gen_range(0..9);
            trace!("Generated digit: {digit}");
            digits.push(char::from_digit(digit, 10).unwrap());
        }

        digits
    }

    async fn get_chars(&self, amount: &usize) -> Vec<char> {
        trace!("Generating {amount} chars");
        let mut str = String::with_capacity(amount.clone());

        if self.rules.separator_matching {
            let char = self.rules.separator_mode.get(&CHARS);
            trace!("Generated char in matching mode: {char}");
            while str.len() < *amount {
                str.push(char.clone());
            }
        } else {
            while str.len() < *amount {
                let char = self.rules.separator_mode.get(&CHARS);
                trace!("Generated char: {char}");
                str.push(char);
            }
        }

        str.chars().collect::<Vec<char>>()
    }
}
