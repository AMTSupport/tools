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
use crate::ui::builder::error::FillResult;
use std::fmt::Debug;
use std::marker::PhantomData;
use std::str::FromStr;

#[derive(Debug, Copy, Clone)]
pub struct FillableDefinition<T>
where
    T: Debug + Sized,
{
    pub name: &'static str,
    pub default: Option<fn() -> T>,
}

type ImplFromStr<T>
where
    T: FromStr + Sized + Debug,
= T;
type ImplBuildable<T>
where
    T: Buildable + Sized + Debug,
= T;

#[derive(Debug)]
pub enum TypeWrapped<T> {
    Bool(FillableDefinition<bool>),
    String {
        pd: PhantomData<T>,
        def: FillableDefinition<ImplFromStr<T>>,
    },
    Buildable {
        pd: PhantomData<T>,
        def: FillableDefinition<ImplBuildable<T>>,
    },
}

#[derive(Debug)]
pub enum TypeWrappedRet<T> {
    Bool(bool),
    String(ImplFromStr<T>, PhantomData<T>),
    Buildable(ImplBuildable<T>, PhantomData<T>),
}

#[derive(Debug, Copy, Clone)]
pub enum PureType {
    Bool,
    FromStr,
    Buildable,
}

#[derive(Debug, Copy, Clone)]
pub struct Fillable<T: ?Sized> {
    /// The name of the field which is being filled.
    pub name: &'static str,

    /// The default value of the field, if provided.
    pub default: Option<fn() -> T>,

    /// A pure type which can be used to fill the field.
    ///
    /// This option can only really be filled by reflection during the proc-macro stage.
    pub pure_type: PureType,

    /// Whether or not the field can be displayed using [`Display`].
    pub can_display: bool,
}

pub trait Filler {
    async fn fill_bool(&mut self, fillable: FillableDefinition<bool>) -> FillResult<TypeWrappedRet<bool>>;

    // async fn fill_choice<T>(&mut self, fillable: Fillable<T>, items: Vec<T>, default: Option<T>) -> FillResult<T>;

    async fn fill_input<T>(
        &mut self,
        fillable: FillableDefinition<T>,
        _pd: PhantomData<T>,
    ) -> FillResult<TypeWrappedRet<T>>
    where
        T: Debug + FromStr;
}
