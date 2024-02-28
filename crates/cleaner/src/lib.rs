/*
 * Copyright (C) 2023-2024. James Draycott me@racci.dev
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
#![allow(incomplete_features)]
#![feature(associated_type_defaults)]
#![feature(const_trait_impl)]
#![feature(downcast_unchecked)]
#![feature(extend_one)]
#![feature(impl_trait_in_assoc_type)]
#![feature(inherent_associated_types)]
#![feature(io_error_more)]
#![feature(lazy_cell)]
#![feature(thin_box)]
#![feature(trait_alias)]
#![feature(type_alias_impl_trait)]

pub mod application;
pub mod cleaners;
pub mod config;
pub mod rule;
