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
use crate::ui::builder::error::{BuildResult, FillResult};
use crate::ui::builder::filler::Filler;
use std::fmt::Debug;

pub mod buildable;
pub mod dummy;
pub mod error;
pub mod filler;

pub trait Builder: Default + Debug {
    type Buildable: Buildable;

    async fn fill<F: Filler>(self, filler: &mut F) -> FillResult<Self>;

    async fn build(self) -> BuildResult<Self::Buildable>;
}
