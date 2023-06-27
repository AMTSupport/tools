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

use tracing::error;
use sysexits::ExitCode;

const ERROR_MESSAGE: &str = "Failed to elevate privileges";

pub fn elevated_privileges() -> bool {
    #[cfg(windows)]
    return is_elevated::is_elevated();

    #[cfg(unix)]
    nix::unistd::geteuid().is_root()
}

pub fn required_elevated_privileges() -> Option<ExitCode> {
    let code = elevated_privileges();

    if !code {
        error!("{}", ERROR_MESSAGE);
        return Some(ExitCode::NoPerm);
    }

    None
}
