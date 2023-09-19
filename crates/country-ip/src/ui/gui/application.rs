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

use crate::gui::pages::generate::{GenerateMessage, GeneratePage};
use crate::gui::pages::lookup::{LookupMessage, LookupPage};
use anyhow::Error;
use iced::{
    widget::{button, column, pick_list, row, text, Container},
    Command, Element, Length, Renderer, Theme,
};
use keshvar::{find_by_iso_short_name, Alpha2, Country, CountryIterator};
use std::borrow::Cow;
use std::fmt::{Display, Formatter};
use std::future::IntoFuture;
use std::iter::IntoIterator;
use std::net::IpAddr;
use std::sync::LazyLock;
use tracing::error;

#[derive(Debug)]
pub struct CountryIPApp {
    selected_country: Option<WrappedCountry>,
    shown_addr: Option<IpAddr>,

    page: usize,
    pages: Vec<Page>,

    error: Option<Error>,
}

#[derive(Debug, Clone)]
pub enum Page {
    Lookup(LookupPage),
    Generate(GeneratePage),
}

#[derive(Debug, Clone)]
pub enum Message {
    Error(String),
    ChangeTab(usize),

    Lookup(LookupMessage),
    Generate(GenerateMessage),
}

impl iced::Application for CountryIPApp {
    type Executor = iced::executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = ();

    fn new(_flags: Self::Flags) -> (Self, Command<Self::Message>) {
        // TODO - load data?
        let instance = Self {
            selected_country: None,
            shown_addr: None,
            page: 0,
            pages: vec![Page::Generate(GeneratePage::default()), Page::Lookup(LookupPage::default())],
            error: None,
        };
        let command = Command::none();

        (instance, command)
    }

    fn title(&self) -> String {
        self.pages[self.page].title()
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        use std::future::IntoFuture;

        match message {
            Message::ChangeTab(page) => {
                self.page = page;
                Command::none()
            }
            Message::Error(err) => {
                error!("Error: {}", err);
                // TODO - show error
                Command::none()
            }
            Message::Lookup(_) => {}
            Message::Generate(_) => {}
        }
    }

    fn view(&self) -> Element<'_, Self::Message, Renderer<Self::Theme>> {
        let content = match &self.pages[self.page] {
            Page::Lookup(page) => page.view(),
            Page::Generate(page) => page.view(),
        };

        Container::new(content).width(Length::Fill).height(Length::Fill).center_x().center_y().into()
    }

    fn theme(&self) -> Self::Theme {
        Theme::Dark
    }
}

pub(crate) const WRAPPED_COUNTRIES: LazyLock<Vec<WrappedCountry>> =
    LazyLock::new(|| CountryIterator::new().into_iter().map(WrappedCountry).collect());

#[derive(Debug, Clone)]
pub struct WrappedCountry(pub Country);

impl PartialEq<Self> for WrappedCountry {
    fn eq(&self, other: &Self) -> bool {
        self.0.country_code() == other.0.country_code()
    }
}

impl Eq for WrappedCountry {}

impl Display for WrappedCountry {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.iso_short_name())
    }
}

impl Into<WrappedCountry> for String {
    fn into(self) -> WrappedCountry {
        WrappedCountry(find_by_iso_short_name(&self).unwrap())
    }
}

impl From<Alpha2> for WrappedCountry {
    fn from(value: Alpha2) -> Self {
        WrappedCountry(value.to_country())
    }
}

impl Into<Cow<'_, WrappedCountry>> for WrappedCountry {
    fn into(self) -> Cow<'static, WrappedCountry> {
        Cow::Owned(self)
    }
}
