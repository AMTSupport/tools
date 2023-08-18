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

use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Copy)]
pub struct Metadata {
    pub mtime: DateTime<Utc>,
    pub size: u64,
    pub is_dir: bool,
    pub is_file: bool,
}

impl From<std::fs::Metadata> for Metadata {
    fn from(value: std::fs::Metadata) -> Self {
        Self {
            mtime: value.modified().unwrap().into(),
            size: value.len(),
            is_dir: value.is_dir(),
            is_file: value.is_file(),
        }
    }
}

#[cfg(feature = "opendal")]
impl From<opendal::Metadata> for Metadata {
    fn from(value: opendal::Metadata) -> Self {
        Self {
            mtime: value.last_modified().unwrap(),
            size: value.content_length(),
            is_dir: value.is_dir(),
            is_file: value.is_file(),
        }
    }
}
