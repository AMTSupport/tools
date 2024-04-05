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

use std::env::consts;

use anyhow::Result;
use self_update::backends::github::Update;

const REPO_OWNER: &str = "AMTSupport";
const REPO_NAME: &str = "tools";

pub async fn update() -> Result<()> {
    const VERSION: &str = env!("CARGO_PKG_VERSION");
    const EXECUTABLE: &str = const_format::formatcp!(
        "{PKG_NAME}-{ARCH}-{PLATFORM}{EXT}",
        PKG_NAME = env!("CARGO_PKG_NAME"),
        ARCH = consts::ARCH,
        PLATFORM = consts::OS,
        EXT = consts::EXE_EXTENSION,
    );

    Update::configure()
        .repo_owner(REPO_OWNER)
        .repo_name(REPO_NAME)
        .bin_name(EXECUTABLE)
        .show_download_progress(true)
        .current_version(VERSION)
        .build()?
        .update()?;

    Ok(())
}
