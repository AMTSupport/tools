use crate::config::separator::{SeparatorMode, CHARS};
use crate::config::transformation::Transformation;
use clap::Parser;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

/// The rules which are used to generate passwords.
#[derive(Parser, Serialize, Deserialize, Debug, Copy, Clone, PartialEq)]
#[serde(default)]
pub struct Rules {
    /// How many words are used.
    #[serde(alias = "words")]
    #[arg(short = 'w', long = "words", default_value_t, value_parser = clap::value_parser!(u16).range(1..10))]
    pub word_count: usize,
    /// The minimum length of each word.
    #[serde(alias = "min_length")]
    #[arg(short = 'm', long = "min-length", default_value_t, value_parser = clap::value_parser!(u16).range(3..9))]
    pub word_length_min: usize,
    /// The maximum length of each word.
    #[serde(alias = "max_length")]
    #[arg(short = 'M', long = "max-length", default_value_t, value_parser = clap::value_parser!(u16).range(3..9))]
    pub word_length_max: usize,
    /// The number of digits to add before the password.
    #[arg(short = 'd', long = "digits-before", default_value_t, value_parser = clap::value_parser!(u16).range(0..))]
    pub digits_before: usize,
    /// The number of digits to add after the password.
    #[arg(short = 'D', long = "digits-after", default_value_t, value_parser = clap::value_parser!(u16).range(0..))]
    pub digits_after: usize,
    /// The transformation to apply to each word.
    #[arg(short = 't', long = "transform", default_value_t = Rules::default().transform, value_enum)]
    pub transform: Transformation,
    /// The separator mode or singular character to use.
    #[serde(alias = "separator_char")]
    #[arg(short = 's', long = "separator-char", default_value_t = Rules::default().separator_mode, value_enum)]
    pub separator_mode: SeparatorMode,
    // /// The list of characters which can be used for the separator.
    // #[arg(short = 'S', long = "separator-alphabet", default_value_t)]
    // pub separator_alphabet: ,
    /// If all separator characters in the password should be the same.
    #[serde(alias = "match_random_char")]
    #[arg(
        short = 'r',
        long = "match-random-char",
        long = "separator_matching",
        default_value_t
    )]
    pub separator_matching: bool,
    /// The number of passwords to generate.
    #[arg(short = 'a', long = "amount", default_value_t)]
    pub amount: usize,
}

impl Default for Rules {
    fn default() -> Self {
        Rules {
            word_count: 2,
            word_length_min: 5,
            word_length_max: 7,
            transform: Transformation::Capitalise,
            separator_mode: SeparatorMode::Random,
            // separator_alphabet: String::from_iter(CHARS),
            separator_matching: true,
            digits_before: 0,
            digits_after: 3,
            amount: 3,
        }
    }
}
