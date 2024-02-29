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

use crate::ui::builder::error::{FillError, FillResult};
use crate::ui::builder::filler::{FillableDefinition, Filler};
use crate::ui::cli::CliUi;
use std::fmt::Debug;
use std::str::FromStr;

impl<C: CliUi> Filler for C {
    async fn fill_bool(&mut self, fillable: &FillableDefinition<bool>) -> FillResult<bool> {
        inquire::Confirm::new(&format!("enter a value for {}", fillable.name))
            .with_placeholder("y/n")
            .with_error_message("invalid input, please enter y/n")
            .with_default(fillable.default.is_some_and(|v| v()))
            .prompt()
            .map_err(|e| FillError::InvalidInput {
                field: fillable.name.to_string(),
                input: e.to_string(),
            })
    }

    // async fn fill_choice<T>(&mut self, fillable: Fillable<T>, items: Vec<T>, default: Option<T>) -> FillResult<T> {
    //     let (items, default) = if fillable.can_display {
    //         (unsafe { transmute_unchecked::<_, Vec<Box<dyn Display>>>(items) },
    //         default.map(|d| unsafe { transmute_unchecked::<_, impl Display>(d) }))
    //     } else {
    //         struct Wrapper<T>(T);
    //         impl <T> Display for Wrapper<T> {
    //             fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    //                 write!(f, "Non-Display Type {}", type_name::<T>())
    //             }
    //         }
    //
    //         (items.into_iter().map(|i| Wrapper(i)).collect(), default.map(|d| Wrapper(d)))
    //     };
    //
    //     inquire::Select::new(&*format!("enter a value for {}", fillable.name), items)
    //         .prompt()
    //         .map_err(|e| FillError::InvalidInput {
    //             field: fillable.name.to_string(),
    //             input: e.to_string(),
    //         })
    //         .into()
    // }

    async fn fill_input<T>(&mut self, fillable: &FillableDefinition<T>) -> FillResult<T>
    where
        T: FromStr + Clone + Debug,
    {
        inquire::Text::new(&format!("enter a value for {}", fillable.name))
            .prompt()
            .map_err(|e| FillError::InvalidInput {
                field: fillable.name.to_string(),
                input: e.to_string(),
            })
            .and_then(|s| {
                s.parse().map_err(|_| FillError::InvalidInput {
                    field: fillable.name.to_string(),
                    input: s.to_string(),
                })
            })
    }
}
