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

use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

#[cfg(test)]
#[allow(unused_imports)]
use fake::{faker::lorem::en::Word, Dummy};

/// The internal identifier for this entity.
#[cfg_attr(test, derive(Dummy))]
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct Id(
    #[serde(default, skip_serializing_if = "String::is_empty")]
    #[cfg_attr(test, dummy(faker = "26"))]
    pub String,
);

/// The user facing & modifiable name for this entity.
#[cfg_attr(test, derive(Dummy))]
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum UniqueName {
    Label(
        #[serde(default, skip_serializing_if = "String::is_empty")]
        #[cfg_attr(test, dummy(faker = "Word()"))]
        String,
    ),
    Name(
        #[serde(default, skip_serializing_if = "String::is_empty")]
        #[cfg_attr(test, dummy(faker = "Word()"))]
        String,
    ),
    Title(
        #[serde(default, skip_serializing_if = "String::is_empty")]
        #[cfg_attr(test, dummy(faker = "Word()"))]
        String,
    ),
}

#[cfg_attr(test, derive(Dummy))]
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct Identifier {
    pub id: Id,

    #[serde(flatten)]
    pub unique_name: UniqueName,
}

impl Identifier {
    /// # Returns
    /// The internal tracking identifier for this entity.
    pub fn id(&self) -> &str {
        self.id.0.as_str()
    }

    /// # Returns
    /// The user facing & modifiable name for this entity.
    pub fn named(&self) -> &str {
        match &self.unique_name {
            UniqueName::Label(label, ..) => label,
            UniqueName::Name(name, ..) => name,
            UniqueName::Title(title, ..) => title,
        }
    }
}

impl Display for Identifier {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{named} ({id})", named = &self.named(), id = &self.id())
    }
}
