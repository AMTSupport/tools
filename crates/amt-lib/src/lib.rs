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

#![feature(associated_type_defaults)]
#![allow(async_fn_in_trait)]
#![allow(incomplete_features)]
#![allow(internal_features)]

pub mod fs;
pub mod helper;
pub mod log;
pub mod macros;
pub mod named;
pub mod pathed;
#[cfg(feature = "ui-cli")]
pub mod ui;
#[cfg(feature = "updater")]
pub mod updater;
