/*
 * Copyright (C) 2024. James Draycott me@racci.dev
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
 * You should have received a copy of the GNU General Public License
 * along with this program. If not, see https://www.gnu.org/licenses/.
 */

#[cfg(feature = "ui-cli")]
pub mod cli;

pub trait Ui<R = anyhow::Result<Self>> {
    /// The arguments that will be used to create the [`Ui`]
    ///
    /// This can be used to pass in configuration options, or other data.
    /// This is not required to be used, but is available if needed.
    type Args = ();

    /// Create a new instance of the [`Ui`]
    ///
    /// This is used to create a new instance of the [`Ui`], and can be used to
    /// parse arguments, or other data.
    ///
    /// Your logging guard should be set up here, as well as any other
    /// configuration that is needed.
    #[allow(clippy::new_ret_no_self)]
    fn new(args: Self::Args) -> R
    where
        Self: Sized;
}
