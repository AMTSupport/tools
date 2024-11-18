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

use crate::ui::builder::error::FillResult;
use std::fmt::Debug;
use std::marker::PhantomData;
use std::str::FromStr;

#[derive(Debug, Clone, Default)]
pub struct FillableDefinition<T>
where
    T: Sized + Clone,
{
    pub name: &'static str,
    pub required: bool,
    pub default: Option<T>,
    pub description: Option<&'static str>,

    pub value: Option<T>,

    pub _pd: PhantomData<T>,
}

pub trait Filler: Debug {
    async fn fill_bool(&self, fillable: &FillableDefinition<bool>) -> FillResult<bool>;

    // async fn fill_choice<T>(&mut self, fillable: Fillable<T>, items: Vec<T>, default: Option<T>) -> FillResult<T>;

    async fn fill_input<T>(&self, fillable: &FillableDefinition<T>) -> FillResult<T>
    where
        T: FromStr + Clone + Debug;
}
