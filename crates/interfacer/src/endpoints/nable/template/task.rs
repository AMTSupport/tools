/*
 * Copyright (c) 2023-2024. James Draycott <me@racci.dev>
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

use self::super::UniqueName;
use std::collections::HashMap;

pub struct TemplateTask {
    /// The unique ID of the task within N-Able.
    id: String,

    /// The arguments that are passed to the task.
    arguments: HashMap<String, String>,

    /// The schedule of the task.
    schedule: TemplateTaskSchedule,

    unique_name: fn(&Self) -> String,
}

impl UniqueName for TemplateTask {
    fn unique_suffix(&self) -> String {
        (self.unique_name)(self)
    }
}

pub struct TemplateTaskSchedule {
    days: u8,
    time: u16,
}
