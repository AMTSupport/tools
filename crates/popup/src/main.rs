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

use iced::alignment::Vertical;
use iced::theme::Palette;
use iced::widget::{button, column, row, text, text_input, Column, Container};
use iced::window::Position;
use iced::window::{Action, Mode};
use iced::{alignment::Horizontal, executor, widget, window, Application, Color, Command, Element, Length, Settings};
use std::process::ExitCode;
use tracing::trace;

#[derive(Default)]
struct Informer {
    message: String,
    requires_confirmation: bool,
    input: String,
}

#[derive(Debug, Clone)]
enum Message {
    Event(String),
    Exit(String),
    UpdateInput(String),
}

impl Application for Informer {
    type Executor = executor::Default;
    type Message = Message;
    type Theme = iced::Theme;
    type Flags = (String, bool);

    fn new(flags: Self::Flags) -> (Self, Command<Message>) {
        use iced_runtime::command::Action as CommandAction;

        let application = Informer {
            message: flags.0,
            requires_confirmation: flags.1,
            ..Default::default()
        };

        let cmd = Command::batch([
            Command::single(CommandAction::Window(Action::GainFocus)),
            Command::single(CommandAction::Window(Action::ChangeMode(Mode::Fullscreen))),
        ]);

        (application, cmd)
    }

    fn title(&self) -> String {
        String::from("Informer - AMT")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::UpdateInput(input) => {
                self.input = input;
                Command::none()
            }
            Message::Exit(reason) => {
                trace!("Exiting: {}", reason);
                window::close()
            }
            Message::Event(msg) => {
                trace!("Event: {}", msg);
                Command::none()
            }
        }
    }

    fn view(&self) -> Element<Message> {
        let content = column![
            text("AMT - Informer").size(18).horizontal_alignment(Horizontal::Center).vertical_alignment(Vertical::Top),
            text("Automated Message from AMT")
                .size(16)
                .horizontal_alignment(Horizontal::Center)
                .vertical_alignment(Vertical::Center),
            "",
            text(self.message.trim()).size(24).horizontal_alignment(Horizontal::Center)
        ];
        // .align_items(Alignment::Center)
        // .height(Length::Shrink)
        // .width(Length::Shrink)
        // .spacing(5)
        // .padding(20);

        let footer = if self.requires_confirmation {
            column![
                text("Please type \"Confirm\" if you understand the this message and wish to close this window.")
                    .size(16)
                    .horizontal_alignment(Horizontal::Center)
                    .vertical_alignment(Vertical::Center),
                text(""),
                text_input("Confirm", &self.input)
                    .on_paste(|p| Message::Event(format!("Ignoring paste: {}", p)))
                    .width(Length::Fixed(100.0))
                    .padding([10, 20])
                    .on_input(|input| {
                        let should_be = "Confirm".to_string();
                        let should_be_trimmed = should_be[0..input.len()].to_string();

                        if input == should_be_trimmed {
                            if input.len() == should_be.len() {
                                Message::Exit("Input is complete and correct".into())
                            } else {
                                Message::UpdateInput(input.clone())
                            }
                        } else {
                            Message::Event("Input was incorrect and hasn't been updated.".into())
                        }
                    }),
            ]
        } else {
            column![button("Close").on_press(Message::Exit("No confirmation required".into()))]
        };
        // .width(Length::Shrink)
        // .height(Length::Shrink)
        // .align_items(Alignment::Center)
        // .spacing(5)
        // .padding(20);
        //

        // let content = column![content]
        //     .align_items(Alignment::Center)
        //     .width(Length::Shrink)
        //     .height(Length::Shrink)
        //     .spacing(5)
        //     .padding(20);

        use iced_aw::{modal, Card};

        let overlay = Card::new(
            widget::Text::new("My Modal").into(),
            widget::Text::new("This is a modal!").into(),
        )
        .foot(footer)
        .max_width(300.0)
        .on_close(Message::Exit("Modal closed".into()));

        modal::Modal::new(content, Some(overlay))
            .backdrop(Message::Event("Backdrop clicked".into()))
            .on_esc(Message::Event("Esc pressed".into()))
            .into()

        // container(content)
        //     .width(Length::FillPortion(30))
        //     .height(Length::FillPortion(30))
        //     .padding(20)
        //     .center_x()
        //     .center_y()
        //     .into()
    }

    fn theme(&self) -> Self::Theme {
        use iced::Theme::{Custom, Dark};

        Custom(Box::from(iced::theme::Custom::new(Palette {
            text: Color::WHITE,
            // primary: Color::from_rgb(0.11, 0.42, 0.87),
            background: Color::from_rgb(0.11, 0.42, 0.87),
            ..Dark.palette()
        })))
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<ExitCode> {
    let _guard = lib::log::init("informer", &Default::default());

    let settings = Settings {
        id: None,
        window: window::Settings {
            level: window::Level::AlwaysOnTop,
            decorations: false,
            resizable: false,
            position: Position::Centered,
            icon: None, // TODO - Add icon
            transparent: true,
            // TODO - Make fullscreen
            ..Default::default()
        },
        flags: (
            r#"
            You will be rebooted at 2 AM;
            please close all work you may have.
            "#
            .trim()
            .to_string(),
            true,
        ),
        exit_on_close_request: false,
        ..Default::default()
    };

    Informer::run(settings)?;
    toast();

    Ok(ExitCode::SUCCESS)
}

#[cfg(windows)]
fn toast() {
    use winrt_notification::{Duration, Sound, Toast};

    Toast::new(Toast::POWERSHELL_APP_ID)
        .title("Look at this flip!")
        .text1("(╯°□°）╯︵ ┻━┻")
        .sound(Some(Sound::SMS))
        .duration(Duration::Short)
        .show()
        .expect("unable to toast");
}

#[cfg(unix)]
fn toast() {
    todo!("not implemented on unix")
}
