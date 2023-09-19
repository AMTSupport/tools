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

use crate::ui::tui::handler::EventHandler;
use ratatui::prelude::CrosstermBackend;
use ratatui::Terminal;
use std::io::Stdout;
use crossterm::event::KeyEvent;
use lib::ui::tui::tui;

struct Tui<'a> {
    terminal: Option<Terminal<CrosstermBackend<Stdout>>>,
    events: EventHandler,

    titles: Vec<&'a str>,
    index: usize,
}

impl tui::Tui for Tui {
    type App = ();
    type Args = ();
    type Events = ();

    fn new(events: Self::Events, args: Self::Args) -> anyhow::Result<Self> {
        todo!()
    }

    fn init(&mut self) -> anyhow::Result<()> {
        todo!()
    }

    fn exit(&mut self) -> anyhow::Result<()> {
        todo!()
    }

    fn draw(&mut self, app: &mut Self::App) -> anyhow::Result<()> {
        todo!()
    }

    fn handle_key_events(&mut self, key_event: KeyEvent, app: &mut Self::App) -> anyhow::Result<()> {
        todo!()
    }

    fn get_terminal(&mut self) -> &mut Terminal<CrosstermBackend<Stdout>> {
        todo!()
    }

    fn set_terminal(&mut self, terminal: Terminal<CrosstermBackend<Stdout>>) -> Option<Terminal<CrosstermBackend<Stdout>>> {
        todo!()
    }
}
