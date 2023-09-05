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
#![feature(const_for)]
#![feature(const_option)]
#![feature(result_option_inspect)]
#![feature(associated_type_defaults)]

pub mod cli;
pub mod fs;
pub mod helper;
pub mod inquire;
pub mod log;
pub mod pathed;
pub mod progress;
pub mod runtime;

