/*
 * Copyright (c) 2024. James Draycott <me@racci.dev>
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
 * You should have received a copy of the GNU General Public License along with this program.
 * If not, see <https://www.gnu.org/licenses/>.
 */

pub mod endpoints;
pub mod structs;
pub mod template;

use anyhow::Result;
use std::str::FromStr;

#[derive(Debug, Clone)]
pub struct NSightApiKey(pub String);

impl NSightApiKey {
    pub fn new<S: AsRef<str>>(key: S) -> Result<Self> {
        if !NSightApiKey::verify(key.as_ref()) {
            anyhow::bail!("Invalid API key provided.");
        }

        Ok(Self(key.as_ref().to_string()))
    }

    pub fn verify<S: AsRef<str>>(key: S) -> bool {
        key.as_ref().len() == 32
    }
}

impl FromStr for NSightApiKey {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        NSightApiKey::new(s)
    }
}
