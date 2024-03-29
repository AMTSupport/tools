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

use chrono::Duration;
use std::sync::LazyLock;
use tracing::{error, instrument};

static MAXIMUM_UPTIME: LazyLock<Duration> = LazyLock::new(|| Duration::minutes(1));

#[instrument(level = "TRACE", ret)]
pub(crate) fn needs_reboot(maximum: Option<&Duration>) -> bool {
    let Ok(Ok(uptime)) = uptime_lib::get()
        .inspect_err(|err| {
            error!("failed to get uptime: {err}");
        })
        .map(Duration::from_std)
    else {
        return false;
    };

    uptime > *maximum.unwrap_or(&MAXIMUM_UPTIME)
}
