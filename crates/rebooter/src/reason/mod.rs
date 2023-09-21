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

mod update;
mod uptime;

use macros::EnumVariants;
use thiserror::Error;

#[derive(Debug, Clone, Error, EnumVariants)]
pub enum Reason {
    /// The system requires a reboot to apply updates cleanly.
    ///
    /// The system may have already applied updates, but the reboot is required to
    /// complete the process.
    ///
    /// This is only available on Windows.
    #[error("The system requires a reboot to apply updates")]
    SystemUpdate,

    /// The system has been up for longer than 7 days.
    #[error("The system has been online for longer than 7 days")]
    Uptime,
}

impl Reason {
    pub fn valid(&self) -> bool {
        match self {
            Self::SystemUpdate => update::needs_reboot(),
            Self::Uptime => uptime::needs_reboot(None),
        }
    }
}
