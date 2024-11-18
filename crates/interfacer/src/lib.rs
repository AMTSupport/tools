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

#![feature(async_closure)]
#![feature(once_cell_try)]

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Deserializer};
use std::str::FromStr;
use tracing::error;

pub mod endpoints;

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

pub fn deserialise_datetime_opt<'de, D>(deserializer: D) -> Result<Option<DateTime<Utc>>, D::Error>
where
    D: Deserializer<'de>,
{
    match String::deserialize(deserializer) {
        Ok(s) if s.is_empty() => Ok(None),
        Ok(s) => match chrono::DateTime::parse_from_rfc3339(&s) {
            Ok(dt) => Ok(Some(dt.with_timezone(&Utc))),
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
        Ok(s) => match NaiveDate::from_str(&s) {
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

pub fn deserialise_date_opt<'de, D>(deserializer: D) -> Result<Option<NaiveDate>, D::Error>
where
    D: Deserializer<'de>,
{
    match String::deserialize(deserializer) {
        Ok(s) if s.is_empty() => Ok(None),
        Ok(s) => match NaiveDate::from_str(&s) {
            Ok(dt) => Ok(Some(dt)),
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
