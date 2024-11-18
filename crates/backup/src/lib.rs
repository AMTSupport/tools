/*
 * Copyright (c) 2024. James Draycott <me@racci.dev>
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

#![feature(path_file_prefix)]
#![feature(type_alias_impl_trait)]
#![feature(trait_alias)]
#![feature(exit_status_error)]
#![feature(unwrap_infallible)]
#![feature(slice_pattern)]
#![feature(let_chains)]
#![feature(thin_box)]
#![feature(async_closure)]
#![feature(const_trait_impl)]
#![feature(result_flattening)]
#![feature(fn_traits)]
#![feature(stmt_expr_attributes)]
#![feature(exact_size_is_empty)]
#![feature(assert_matches)]
#![feature(core_intrinsics)]
#![allow(internal_features)]
#![allow(async_fn_in_trait)]

pub mod config;
pub mod sources;
pub mod ui;
