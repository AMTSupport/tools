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

pub mod runners;
pub mod structs;
pub mod web;

pub const API_ENDPOINT: &str = "/api/v1";
pub const API_HEADER: &str = "X-Api-Key";
pub const COMPANIES_ENDPOINT: &str = "/companies";
pub const PASSWORDS_ENDPOINT: &str = "/asset_passwords";

#[derive(clap::Subcommand)]
pub enum HuduCommands {
    Query {
        #[arg(short, long)]
        outdated_passwords: bool,
    },
}
