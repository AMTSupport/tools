/*
 * Copyright (c) 2023. James Draycott <me@racci.dev>
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, version 3.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License along with this program.
 * If not, see <https://www.gnu.org/licenses/>.
 */

use std::marker::PhantomData;
use thiserror::Error;

#[derive(Error)]
pub enum BuildableError<T> {
    #[error("unable to fill {} from input {}: {}", name, input, std::any::type_name::<T>())]
    UnableToFill {
        name: String,

        input: String,

        _ty: PhantomData<T>,
    },

    #[error("unable to build {name} from builder")]
    UnableToBuild {
        name: String,

        #[source]
        source: anyhow::Error,
    },
}
