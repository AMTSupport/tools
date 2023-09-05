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

use cfg_if::cfg_if;
use sysexits::ExitCode;
use tracing::error;

const ERROR_MESSAGE: &str = "Failed to elevate privileges";

pub fn elevated_privileges() -> bool {
    cfg_if! {
         if #[cfg(windows)] {
            is_elevated::is_elevated()
        } else if #[cfg(unix)] {
            rustix::process::getegid().is_root()
        } else {
            warn!("Unsupported platform, assuming not elevated");
            false
        }
    }
}

pub fn require_elevated_privileges() -> Option<ExitCode> {
    let code = elevated_privileges();

    if !code {
        error!("{}", ERROR_MESSAGE);
        return Some(ExitCode::NoPerm);
    }

    None
}
