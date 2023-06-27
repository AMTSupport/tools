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

use clap::Parser;
use cleaner::application::{self, application};
use lib::anyhow::Result;
use lib::helper::required_elevated_privileges;
use lib::log as Logger;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = application::Cli::try_parse()?;
    Logger::init(env!["CARGO_PKG_NAME"], &cli.flags)?;
    let _ = required_elevated_privileges().is_some_and(|code| code.exit());

    application(cli).await?;

    Ok(())
}
