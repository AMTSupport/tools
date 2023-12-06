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

use anyhow::Result;
use std::fmt::Display;

pub trait Filler {
    async fn fill_bool(&mut self, name: &str, default: Option<bool>) -> Result<bool>;

    async fn fill_choice<D>(&mut self, name: &str, items: Vec<D>, default: Option<D>) -> Result<D>
    where
        D: Display;

    async fn fill_input<D>(&mut self, name: &str, default: Option<D>) -> Result<D>
    where
        D: Display;
}
