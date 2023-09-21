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

use crate::reason::Reason;
use anyhow::Result;
use chrono::{DateTime, Duration, Local, NaiveDateTime};
use planif::enums::TaskCreationFlags;
use planif::schedule_builder::ScheduleBuilder;
use planif::settings::{Compatibility, IdleSettings, NetworkSettings, PrincipalSettings, Settings};
use std::ops::Add;
use windows::Win32::System::TaskScheduler;

#[derive(Debug)]
pub struct ScheduledReboot {
    reason: Reason,
    when: DateTime<Local>,
}

impl ScheduledReboot {
    pub fn new(reason: Reason, when: NaiveDateTime) -> Self {
        Self { reason, when }
    }

    /// Schedule a task that will reboot the system at the given time.
    #[cfg(windows)]
    pub fn submit(&self) -> Result<()> {
        use planif::schedule::Schedule;
        use planif::schedule_builder::{Action, ScheduleBuilder, Time};

        let sb = ScheduleBuilder::new()?
            .create_time()
            .author("Rebooter")?
            .description(&format!("Rebooter: {}", self.reason))?
            .trigger("Time Trigger", true)?
            .start_boundary(self.when)?
            .settings(Settings {
                delete_expired_task_after: Some(self.when.add(Duration::minutes(30))),
                wake_to_run: Some(true),
                stop_if_going_on_batteries: Some(false),
            })
            .build()?
            .register("Rebooter", TaskCreationFlags::CreateOrUpdate)?;
    }
}
