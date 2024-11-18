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

use anyhow::Result;
use lib::ui::{cli::CliUi, Ui};
use rebooter::ui::cli::RebooterCli;
use sysexits::ExitCode;

#[tokio::main]
async fn main() -> Result<ExitCode> {
    if let Some(code) = lib::helper::require_elevated_privileges() {
        return Ok(code);
    }

    let mut application = RebooterCli::new(())?;
    application.run().await?;

    Ok(ExitCode::Ok)
}
