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

use crate::db_ip;
use crate::gui::application::WrappedCountry;
use crate::gui::page::Page;
use iced::application::StyleSheet;
use iced::widget::{button, column, text, text_input, Container};
use iced::{Command, Renderer};
use keshvar::Alpha2;
use std::future::IntoFuture;
use std::net::IpAddr;

#[derive(Debug, Clone)]
pub enum LookupMessage {
    Result(Option<WrappedCountry>),
    InputChanged(String),
    Lookup,
}

#[derive(Default, Debug, Clone)]
pub struct LookupPage {
    pub addr: Option<IpAddr>,

    pub input: String,
    pub valid: bool,

    pub result: Option<WrappedCountry>,
}

impl Page for LookupPage {
    type Message = LookupMessage;

    fn title(&self) -> String {
        "IP Lookup".to_string()
    }

    fn view<T>(&mut self) -> Container<'_, Self::Message, T> {
        let content = column![
            text("IP Address").size(12),
            text_input(&mut self.input, "IP Address", LookupMessage::InputChanged),
            button("Lookup").on_press(LookupMessage::Lookup),
            text("Result").size(12),
            text(&self.result.map(|country| country.0.iso_short_name()).unwrap_or_else(|| "None")).size(12),
        ];

        Container::new(content)
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match message {
            LookupMessage::Result(result) => {
                self.result = result;
                Command::none()
            }
            LookupMessage::Lookup => {
                let db = db_ip::DB::instance();
                let addr = match &self.addr {
                    Some(addr) => addr.clone(),
                    None => return Command::none(),
                };

                Command::perform(
                    async move { db.lookup(&*addr).await }.into_future(),
                    |result: Option<Alpha2>| LookupMessage::Result(result.map(WrappedCountry::from)),
                )
            }
            LookupMessage::InputChanged(input) => {
                self.input = input;
                match input.parse::<IpAddr>() {
                    Ok(_) => self.valid = true,
                    Err(_) => self.valid = false,
                };

                Command::none()
            }
        }
    }
}
