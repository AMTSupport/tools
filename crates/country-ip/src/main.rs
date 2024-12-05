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

#[tokio::main]
#[cfg(feature = "ui-cli")]
async fn main() -> amt_lib::ui::cli::CliResult<()> {
    use amt_lib::ui::cli::CliUi;
    use amt_lib::ui::Ui;
    use country_ip::ui::cli::ui::CountryIPCli;

    let mut ui = CountryIPCli::new(())?;
    ui.run().await
}

// #[tokio::main]
// #[cfg(feature = "ui-gui")]
// async fn main() {
//     use country_ip::ui::gui::application::CountryIPApp;
//     use iced::{Application, Sandbox, Settings};
//
//     <CountryIPApp as Application>::run(Settings::default())
// }
