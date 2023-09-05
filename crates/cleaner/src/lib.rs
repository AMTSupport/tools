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
#![feature(lazy_cell)]
#![feature(result_option_inspect)]
#![feature(return_position_impl_trait_in_trait)]
#![feature(type_alias_impl_trait)]
#![feature(trait_alias)]
#![feature(downcast_unchecked)]
#![feature(const_trait_impl)]
#![feature(impl_trait_in_assoc_type)]
#![feature(thin_box)]
#![feature(associated_type_defaults)]
#![feature(extend_one)]
#![feature(io_error_more)]
#![feature(inherent_associated_types)]

pub mod application;
pub mod cleaners;
pub mod config;
pub mod rule;
