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

use std::sync::mpsc::{Receiver, Sender};
use std::thread::JoinHandle;
use lib::ui::tui::event;
use lib::ui::tui::event::Event;

pub struct EventHandler {
    sender: Sender<Event>,
    receiver: Receiver<Event>,
    handler: JoinHandle<()>,
}

impl event::EventHandler for EventHandler {
    const TICK_RATE: u16 = 200;

    fn inner(sender: Sender<Event>, receiver: Receiver<Event>, handler: JoinHandle<()>) -> Self where Self: Sized {
        Self {
            sender,
            receiver,
            handler,
        }
    }

    fn sender(self) -> Sender<Event> {
        self.sender
    }

    fn receiver(self) -> Receiver<Event> {
        self.receiver
    }

    fn handler(self) -> JoinHandle<()> {
        self.handler
    }
}
