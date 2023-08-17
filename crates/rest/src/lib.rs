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

extern crate core;

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Deserializer};
use std::hash::Hash;
use std::str::FromStr;
use tracing::error;

// pub mod hudu;
// pub mod manager;
pub mod nable;

const AGENT: &str = "rest_agent";

#[derive(Debug, Clone)]
pub struct Client {
    base_url: String,
    api_key: String,
    client: reqwest_middleware::ClientWithMiddleware,
}

impl Hash for Client {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.base_url.hash(state);
        self.api_key.hash(state);
    }
}

impl PartialEq<Self> for Client {
    fn eq(&self, other: &Self) -> bool {
        self.base_url == other.base_url && self.api_key == other.api_key
    }
}
//
impl Eq for Client {}

pub fn deserialise_datetime<'de, D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
where
    D: Deserializer<'de>,
{
    match String::deserialize(deserializer) {
        Ok(s) => match chrono::DateTime::parse_from_rfc3339(&s) {
            Ok(dt) => Ok(dt.with_timezone(&Utc)),
            Err(e) => {
                error!("Failed to parse datetime: {:?}", e);
                Err(serde::de::Error::custom("Failed to parse datetime"))?
            }
        },
        Err(e) => {
            error!("Failed to deserialise datetime: {:?}", e);
            Err(serde::de::Error::custom("Failed to deserialise datetime"))?
        }
    }
}

pub fn deserialise_date<'de, D>(deserializer: D) -> Result<NaiveDate, D::Error>
where
    D: Deserializer<'de>,
{
    match String::deserialize(deserializer) {
        Ok(s) => match chrono::NaiveDate::from_str(&s) {
            Ok(dt) => Ok(dt),
            Err(e) => {
                error!("Failed to parse date: {:?}", e);
                Err(serde::de::Error::custom("Failed to parse date"))?
            }
        },
        Err(e) => {
            error!("Failed to deserialise date: {:?}", e);
            Err(serde::de::Error::custom("Failed to date date"))?
        }
    }
}

pub trait Url<C: ?Sized> {
    fn link(&self, client: &C) -> String;
}
