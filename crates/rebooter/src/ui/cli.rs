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

use lib::cli::Flags as CommonFlags;
use lib::ui::cli::oneshot::OneshotHandler;
use lib::ui::cli::{CliResult, CliUi};
use lib::ui::Ui;
use std::fmt::Debug;

pub struct RebooterCli;

impl Ui for RebooterCli {
    fn new(args: Self::Args) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        todo!()
    }
}

impl CliUi for RebooterCli {}

impl OneshotHandler for RebooterCli {
    type Action = ();

    async fn handle(&mut self, command: Self::Action, flags: &CommonFlags) -> CliResult<()> {
        unimplemented!()
    }
}
