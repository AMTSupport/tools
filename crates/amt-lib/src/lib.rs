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
#![feature(async_closure)]
#![feature(auto_traits)]
#![feature(cfg_match)]
#![feature(const_for)]
#![feature(const_format_args)]
#![feature(core_intrinsics)]
#![feature(fn_traits)]
#![feature(impl_trait_in_assoc_type)]
#![feature(let_chains)]
#![feature(negative_impls)]
#![feature(result_flattening)]
#![feature(stmt_expr_attributes)]
#![feature(trait_alias)]
#![feature(trivial_bounds)]
#![feature(try_trait_v2)]
#![feature(type_alias_impl_trait)]
#![feature(negative_bounds)]
#![feature(lazy_type_alias)]
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
