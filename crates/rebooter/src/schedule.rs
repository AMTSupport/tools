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
use chrono::{DateTime, Local};

#[derive(Debug)]
pub struct ScheduledReboot {
    reason: Reason,
    when: DateTime<Local>,
}

impl ScheduledReboot {
    pub fn new(reason: Reason, when: DateTime<Local>) -> Self {
        Self { reason, when }
    }

    /// Schedule a task that will reboot the system at the given time.
    #[cfg(windows)]
    pub fn submit(&self) -> Result<()> {
        use planif::enums::TaskCreationFlags;
        use planif::schedule_builder::{Action, ScheduleBuilder};
        use planif::settings::Settings;

        let sb = ScheduleBuilder::new()?
            .create_time()
            .author("Rebooter")?
            .description(&format!("Rebooter: {}", self.reason))?
            .trigger("Time Trigger", true)?
            .start_boundary(&*self.when.format("%Y-%M-%DT%H:%M:%S").to_string())?
            .settings(Settings {
                allow_demand_start: Some(true),
                allow_hard_terminate: Some(false),
                compatibility: None,
                delete_expired_task_after: Some("P30M".into()),
                wake_to_run: Some(true),
                stop_if_going_on_batteries: Some(false),
                run_only_if_idle: Some(false),
                run_only_if_network_available: Some(false),
                disallow_start_if_on_batteries: Some(false),
                enabled: Some(true),
                execution_time_limit: None,
                hidden: Some(false),
                idle_settings: None,
                multiple_instances_policy: None,
                network_settings: None,
                priority: None,
                restart_count: None,
                restart_interval: None,
                start_when_available: Some(true),
                xml_text: None,
            })?
            .action(Action::new(
                "exec",
                "shutdown.exe",
                "C:\\Windows\\System32",
                &format!("/r /t 0 /c \"{}\"", self.reason),
            ))?
            .build()?
            .register("Rebooter", TaskCreationFlags::CreateOrUpdate.into())?;
    }

    #[cfg(not(windows))]
    pub fn submit(&self) -> Result<()> {
        unimplemented!("Scheduling is not supported on this platform")
    }
}
