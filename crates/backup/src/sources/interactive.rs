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

use crate::config::runtime::RuntimeConfig;
use async_trait::async_trait;
use anyhow::Result;

#[async_trait]
pub trait Interactive<T> {
    /// Creates a new async function which will prompt the user for the required information to create the exporter;
    async fn interactive(config: &RuntimeConfig) -> Result<T>;
}
