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

use crate::impls_utils;
use crate::ui::builder::buildable::Buildable;
use crate::ui::builder::error::FillResult;
use std::fmt::Debug;
use std::intrinsics::transmute_unchecked;
use std::marker::PhantomData;
use std::mem::transmute;
use std::str::FromStr;
use tracing::{debug, info};

#[derive(Debug, Clone, Default)]
pub struct FillableDefinition<T>
where
    T: Sized + Clone,
{
    pub name: &'static str,
    pub default: Option<fn() -> T>,
    pub required: bool,

    pub _pd: PhantomData<T>,
}

type ImplFromStr<T>
where
    T: FromStr + Sized + Clone + Debug,
= T;
type ImplBuilder<T>
where
    T: Buildable + Sized + Clone + Debug,
= T;
type ImplBuildable<T>
where
    T: Buildable + Sized + Clone + Debug,
= T;

#[derive(Debug, Clone)]
pub enum TypeWrapped<T: Sized + Clone + Debug> {
    Bool(FillableDefinition<bool>),
    String(FillableDefinition<ImplFromStr<T>>),
    Buildable(FillableDefinition<ImplBuildable<T>>),
}

impl<T> TypeWrapped<T>
where
    T: Sized + Clone + Debug,
{
    pub fn new(fillable_definition: FillableDefinition<T>) -> Self {
        // Figure out what type we are dealing with by checking the type of T.
        match std::any::type_name::<T>() {
            "bool" => Self::Bool(unsafe { transmute(fillable_definition) }), // This is safe because we are only changing the type of the enum.
            _ => {
                debug!("Unable to decide type, trying to find impl for FromStr");
                debug!("fillable_definition: {:?}", fillable_definition);

                let pd = fillable_definition._pd;
                let str_def = fillable_definition.clone();
                match impls_utils!(|pd, str_def: FillableDefinition<T>| FromStr => TypeWrapped<T> => {
                    Ok(TypeWrapped::String(str_def))
                }) {
                    Ok(v) => return v,
                    Err(_) => {
                        info!("Failed to find impl for FromStr, trying Buildable");
                    }
                }

                match impls_utils!(|pd, fillable_definition: FillableDefinition<T>| Buildable => TypeWrapped<T> => {
                    Ok(TypeWrapped::Buildable(fillable_definition))
                }) {
                    Ok(v) => return v,
                    Err(_) => {
                        info!("Failed to find impl for Buildable.");
                    }
                }

                panic!("No impl found for type");
            }
        }
    }

    pub fn value(&self) -> &FillableDefinition<T> {
        match self {
            // Safety: We are only changing the type of the enum, it was ensured as a bool in the match statement.
            Self::Bool(def) => unsafe { transmute_unchecked(def) },
            Self::String(def) => def,
            Self::Buildable(def) => def,
        }
    }
}

pub trait Filler: Debug {
    async fn fill_bool(&mut self, fillable: &FillableDefinition<bool>) -> FillResult<bool>;

    // async fn fill_choice<T>(&mut self, fillable: Fillable<T>, items: Vec<T>, default: Option<T>) -> FillResult<T>;

    async fn fill_input<T>(&mut self, fillable: &FillableDefinition<T>) -> FillResult<T>
    where
        T: FromStr + Clone + Debug;
}
