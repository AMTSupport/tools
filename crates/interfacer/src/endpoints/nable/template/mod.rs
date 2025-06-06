/*
 * Copyright (C) 2023-2024. James Draycott me@racci.dev
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
 * You should have received a copy of the GNU General Public License
 * along with this program. If not, see https://www.gnu.org/licenses/.
 */

pub mod check;
pub mod task;

use amt_lib::named::Named;

pub struct Template {
    common_name: String,
    checks: Vec<check::TemplateCheck>,
    tasks: Vec<task::TemplateTask>,
}

pub trait UniqueName: Named {
    fn unique_name(&self) -> String {
        format!("{}-{}", <Self as Named>::NAME, self.unique_suffix())
    }

    fn unique_suffix(&self) -> String;
}
