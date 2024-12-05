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

#![feature(async_closure)]
#![feature(cfg_match)]
#![feature(impl_trait_in_assoc_type)]
#![allow(incomplete_features)]
#![feature(inherent_associated_types)]

use amt_lib::helper::require_elevated_privileges;
use anyhow::Result;
use std::sync::LazyLock;
use sys_cleaner::application::application;
use sys_cleaner::config::runtime::Runtime;

static RUNTIME: LazyLock<Runtime> = LazyLock::new(|| Runtime::new().unwrap());

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<()> {
    let _guard = amt_lib::log::init(env!("CARGO_PKG_NAME"), &RUNTIME.cli.flags);
    let _ = require_elevated_privileges().is_some_and(|code| code.exit());
    application(&RUNTIME).await?;

    Ok(())
}
