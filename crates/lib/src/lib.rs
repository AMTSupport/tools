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

#![feature(lazy_cell)]
#![feature(const_for)]
#![feature(const_option)]
#![feature(auto_traits)]
#![feature(negative_impls)]
#![feature(associated_type_defaults)]
#![feature(trait_alias)]
#![feature(trivial_bounds)]
#![feature(stmt_expr_attributes)]
#![feature(cfg_match)]
#![feature(const_format_args)]
#![feature(impl_trait_in_assoc_type)]
#![feature(type_alias_impl_trait)]
#![feature(let_chains)]
#![feature(try_trait_v2)]
#![feature(core_intrinsics)]
#![feature(result_flattening)]
#![feature(async_closure)]
#![feature(fn_traits)]

pub mod cli;
pub mod fs;
pub mod helper;
pub mod log;
pub mod macros;
pub mod named;
pub mod pathed;
#[cfg(any(feature = "ui-gui", feature = "ui-tui", feature = "ui-cli"))]
pub mod ui;
#[cfg(feature = "updater")]
pub mod updater;
