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

use crate::gui::application::{Message, WrappedCountry, WRAPPED_COUNTRIES};
use crate::gui::page::Page;
use iced::application::StyleSheet;
use iced::widget::{button, column, pick_list, row, text, Container};
use iced::{Command, Renderer};
use std::future::IntoFuture;
use std::net::IpAddr;
use tracing::error;

#[derive(Debug, Clone)]
pub enum GenerateMessage {
    UpdateCountry(WrappedCountry),
    UpdateAddr(IpAddr),

    GenerateIpV4,
    GenerateIpV6,

    None,
}

#[derive(Default, Debug, Clone)]
pub struct GeneratePage {
    pub addr: Option<IpAddr>,
    pub country: Option<WrappedCountry>,
}

impl Page for GeneratePage {
    type Message = GenerateMessage;

    fn title(&self) -> String {
        "Generate IP".to_string()
    }

    fn view<C: Container<M>, M>(&mut self) -> C {
        let countries = WRAPPED_COUNTRIES.clone();

        let content = row![
            column![
                text("Country").size(12),
                pick_list(countries, self.country.clone(), GenerateMessage::UpdateCountry),
            ],
            "",
            "",
            column![
                text("Generated Address").size(12),
                text(&self.addr.map(|addr| addr.to_string()).unwrap_or_else(|| "None".to_string())).size(12),
            ],
            column![
                button(text("Generate IPv4"))
                    .on_press(GenerateMessage::GenerateIpV4)
                    .padding(10),
                button(text("Generate IPv6"))
                    .on_press(GenerateMessage::GenerateIpV6)
                    .padding(10),
            ],
        ];

        Container::new(content)
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match message {
            GenerateMessage::None => Command::none(),
            GenerateMessage::UpdateCountry(country) => {
                self.country = Some(country);
                Command::none()
            }
            GenerateMessage::UpdateAddr(addr) => {
                self.addr = Some(addr);
                Command::none()
            }
            GenerateMessage::GenerateIpV4 => {
                let country = match &self.country {
                    Some(country) => country.clone(),
                    None => {
                        error!("No country selected");
                        return Command::none();
                    }
                };
                let alpha = country.0.alpha2();

                Command::perform(
                    async move {
                        match crate::get_record_db(&country.0).await {
                            Ok(db) => db
                                .random_ipv4(&alpha)
                                .await
                                .ok_or_else(|| anyhow::Error::msg("No IPv4 addresses found for country")),
                            Err(err) => Err(err),
                        }
                    }
                    .into_future(),
                    |result| match result {
                        Err(err) => {
                            error!("{}", err.to_string());
                            GenerateMessage::None
                        }
                        Ok(addr) => GenerateMessage::UpdateAddr(addr),
                    },
                )
            }
            GenerateMessage::GenerateIpV6 => {
                let country = match &self.country {
                    Some(country) => country.clone(),
                    None => {
                        error!("No country selected");
                        return Command::none();
                    }
                };
                let alpha = country.0.alpha2();

                Command::perform(
                    async move {
                        match crate::get_record_db(&country.0).await {
                            Ok(db) => db
                                .random_ipv6(&alpha)
                                .await
                                .ok_or_else(|| anyhow::Error::msg("No IPv6 addresses found for country")),
                            Err(err) => Err(err),
                        }
                    }
                    .into_future(),
                    |result| match result {
                        Ok(addr) => GenerateMessage::UpdateAddr(addr),
                        Err(err) => {
                            error!("Error generating IPv6 address: {}", err);
                            GenerateMessage::None
                        }
                    },
                )
            }
        }
    }
}
