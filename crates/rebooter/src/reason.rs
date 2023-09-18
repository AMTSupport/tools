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
use anyhow::{Context, Result};
use chrono::{Duration, Local};
use std::fmt::{Display, Formatter};
use tracing::instrument;

#[derive(Debug, Clone)]
pub enum RebootReason {
    SystemUpdate,
    Uptime(Duration, Duration),
    Custom(String),
}

impl Display for RebootReason {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let msg = match self {
            Self::SystemUpdate => "System updates require a reboot.".to_string(),
            Self::Uptime(uptime, maximum) => format!("Uptime of {uptime} exceeds {maximum}"),
            Self::Custom(message) => message.clone(),
        };

        write!(f, "{}", msg)
    }
}

#[instrument(ret, err)]
pub fn reason_uptime(maximum: Duration) -> Result<Option<RebootReason>> {
    let uptime = uptime_lib::get().ok().context("failed to get uptime")?;
    let uptime = Local::now().timestamp() - uptime.as_secs();
    let uptime = Duration::seconds(uptime);

    if uptime > maximum {
        return Ok(Some(RebootReason::Uptime(uptime, maximum)));
    }

    Ok(None)
}

#[cfg(windows)]
#[instrument(ret, err)]
pub fn reason_system_updates() -> Result<Option<RebootReason>> {
    use registry::Hive;
    use registry::Security;
    use tracing::debug;

    let regkey = Hive::LocalMachine.open(
        r"SOFTWARE\Microsoft\Windows\CurrentVersion\WindowsUpdate\Auto Update",
        Security::Read,
    )?;

    if let Ok(_) = regkey.value("RebootRequired") {
        return Ok(Some(RebootReason::SystemUpdate));
    }

    Ok(None)
}

#[cfg(unix)]
#[instrument(ret, err)]
pub fn reason_system_updates() -> Result<Option<RebootReason>> {
    Ok(None)
}

