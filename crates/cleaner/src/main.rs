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

#![feature(async_closure)]
#![feature(lazy_cell)]

use anyhow::Result;
use cleaner::application::application;
use cleaner::config::runtime::Runtime;
use lib::helper::require_elevated_privileges;
use std::sync::LazyLock;

static RUNTIME: LazyLock<Runtime> = LazyLock::new(|| Runtime::new().unwrap());

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<()> {
    let _guard = lib::log::init(env!("CARGO_PKG_NAME"), RUNTIME.cli.flags.verbose);
    let _ = require_elevated_privileges().is_some_and(|code| code.exit());
    application(&RUNTIME).await?;

    Ok(())
}
