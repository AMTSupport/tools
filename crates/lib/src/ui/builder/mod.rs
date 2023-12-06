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

use crate::ui::builder::buildable::Buildable;
use crate::ui::builder::filler::Filler;

mod buildable;
mod dummy;
mod error;
mod filler;

#[doc(hidden)]
pub(crate) type Result<T, B> = std::result::Result<T, error::BuildableError<B>>;

pub trait Builder: Default {
    type Buildable: Buildable;

    async fn fill<F: Filler>(self, filler: &mut F) -> Result<Self, Self::Buildable>;

    async fn build(self) -> Result<Self::Buildable, Self::Buildable>;
}
