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

#[tokio::main]
#[cfg(feature = "ui-cli")]
async fn main() -> lib::ui::cli::cli::CliResult<()> {
    use country_ip::ui::cli::ui::CountryIPCli;
    use lib::ui::cli::cli::{AsyncCliUI, CliUI};

    let mut ui = CountryIPCli::new(())?;
    ui.run().await
}

#[tokio::main]
#[cfg(feature = "ui-tui")]
async fn main() {
    todo!("tui not implemented yet")
}

#[tokio::main]
#[cfg(feature = "ui-gui")]
async fn main() {
    use country_ip::ui::gui::application::CountryIPApp;
    use iced::{Application, Sandbox, Settings};

    <CountryIPApp as Application>::run(Settings::default())
}
