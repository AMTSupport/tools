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

use tracing::instrument;

#[cfg(windows)]
#[instrument(level = "TRACE", ret)]
pub(crate) fn needs_reboot() -> bool {
    use registry::Hive;
    use registry::Security;

    let regkey = Hive::LocalMachine.open(
        r"SOFTWARE\Microsoft\Windows\CurrentVersion\WindowsUpdate\Auto Update",
        Security::Read,
    )?;

    regkey.value("RebootRequired").is_ok()
}

#[cfg(not(windows))]
#[instrument(level = "TRACE", ret)]
pub(crate) fn needs_reboot() -> bool {
    false
}
