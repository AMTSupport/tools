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

use anyhow::Result;
use lib::ui::Ui;

#[cfg(feature = "ui-tui")]
#[cfg(not(feature = "ui-cli"))]
#[tokio::main(flavor = "multi_thread")]
pub async fn main() -> Result<()> {
    use backup::ui::tui::{event::EventHandler, tui::Tui};

    let event_handler = EventHandler::new(50);
    let mut tui = Tui::new(event_handler)?;
    tui.init()?;

    Ok(())
}

#[cfg(feature = "ui-cli")]
#[cfg(not(feature = "ui-tui"))]
#[tokio::main(flavor = "multi_thread")]
pub async fn main() -> Result<()> {
    use backup::ui::cli::ui::BackupCli;

    let mut app = BackupCli::new(())?;
    app.run().await?;
    Ok(())
    // TODO :: Verify writable
    // TODO :: Verify enough space
    // TODO :: Verify dir is either empty, or has existing backup data
}

#[cfg(not(any(feature = "ui-cli", feature = "ui-tui")))]
pub fn main() -> ! {
    unimplemented!("At least one UI is required.")
}
