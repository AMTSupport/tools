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

use iced::application::StyleSheet;
use iced::widget::Container;
use iced::{Application, Command, Renderer as R};

pub trait Page<'a, Message, Renderer = R> {
    type Message;

    fn title(&self) -> String;

    fn view<C: <M>, M>(&mut self) -> C;

    fn update(&mut self, message: Self::Message) -> Command<Self::Message>;
}
