/*
 * Copyright (C) 2023 James Draycott <me@racci.dev>
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, version 3.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */

#![feature(path_file_prefix)]
#![feature(type_alias_impl_trait)]
#![feature(trait_alias)]
#![feature(exit_status_error)]
#![feature(unwrap_infallible)]
#![feature(slice_pattern)]
#![feature(let_chains)]
#![feature(return_position_impl_trait_in_trait)]
#![feature(result_option_inspect)]
#![feature(thin_box)]
#![feature(async_closure)]
#![feature(file_create_new)]
#![feature(const_trait_impl)]
#![feature(lazy_cell)]
#![feature(result_flattening)]
#![feature(async_fn_in_trait)]
#![feature(fn_traits)]
#![feature(stmt_expr_attributes)]
#![feature(exact_size_is_empty)]

pub mod config;
pub mod sources;
pub mod ui;
