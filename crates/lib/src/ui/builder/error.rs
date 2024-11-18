/*
 * Copyright (c) 2023-2024. James Draycott <me@racci.dev>
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

use crate::ui::builder::buildable::Buildable;
use std::marker::PhantomData;
use thiserror::Error;

pub type FillResult<T> = Result<T, FillError>;
pub type BuildResult<T> = Result<T, BuildError>;
pub type FinalResult<B: Buildable> = Result<B, Error<B>>;

#[derive(Debug, Error)]
pub enum Error<B: Buildable> {
    Fill {
        #[source]
        err: FillError,

        _buildable: PhantomData<B>,
    },

    Buildable {
        #[source]
        err: BuildError,

        _buildable: PhantomData<B>,
    },
}

#[derive(Debug, Error)]
pub enum FillError {
    #[error("unable to fill {field} from input, invalid input: {input}")]
    InvalidInput { field: String, input: String },

    #[error("unable to fill {field} from filler {filler}")]
    InvalidFiller { field: String, filler: String },

    #[error("no fillers available for {field}")]
    NoFillers { field: String },

    #[error("no value found for {field}")]
    NoValue { field: String },

    #[error("Nested builder error: {0}")]
    Nested(#[from] BuildError),

    #[error("unknown error: {0}")]
    Unknown(#[from] anyhow::Error),

    #[error("invalid type for call")]
    InvalidDefinition,
}

#[derive(Debug, Error)]
pub enum BuildError {
    #[error("missing required field {field}")]
    MissingField { field: String },

    #[error("unknown error: {0}")]
    Unknown(#[from] anyhow::Error),
}
