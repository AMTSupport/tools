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

use crate::schedule::ScheduledReboot;
use crate::ui::actions::Action;
use anyhow::Result;
use chrono::{Local, NaiveDateTime, NaiveTime};
use lib::cli::Flags as CommonFlags;
use lib::populate;
use lib::ui::cli::oneshot::OneshotHandler;
use lib::ui::cli::{CliResult, CliUi};
use lib::ui::Ui;
use tracing::{error, info};
use tracing_appender::non_blocking::WorkerGuard;

pub struct RebooterCli {
    _guard: Option<WorkerGuard>,
}

impl Ui for RebooterCli {
    fn new(_args: Self::Args) -> Result<Self>
    where
        Self: Sized,
    {
        Ok(Self { _guard: None })
    }
}

impl CliUi for RebooterCli {}

impl OneshotHandler for RebooterCli {
    type Action = Action;

    async fn handle(&mut self, command: Self::Action, flags: &CommonFlags) -> CliResult<()> {
        populate!(self, flags);

        match command {
            Action::Query { reasons } => {
                let (valid, invalid): (Vec<_>, Vec<_>) = reasons.into_iter().partition(|reason| reason.valid());

                for reason in valid {
                    info!("{reason} would require a restart.");
                }

                for reason in invalid {
                    info!("{reason} would not require a restart.");
                }
            }
            Action::QueryAndSchedule { mut reasons } => {
                while let Some(reason) = reasons.pop() {
                    if !reason.valid() {
                        continue;
                    }

                    let when = get_schedule_time();
                    info!("Scheduling {reason} for {when}.");

                    if let Err(err) = ScheduledReboot::new(reason, when).submit() {
                        error!("Failed to schedule reboot: {err}");
                    }
                }
            }
        }

        Ok(())
    }
}

fn get_schedule_time() -> NaiveDateTime {
    let now = Local::now();

    let reboot_time = NaiveTime::from_hms_opt(3, 0, 0).expect("Invalid time");
    let reboot_date = if now.time().lt(&reboot_time) {
        now.date_naive()
    } else {
        now.date_naive().succ_opt().expect("Invalid date")
    };

    NaiveDateTime::new(reboot_date, reboot_time)
}
