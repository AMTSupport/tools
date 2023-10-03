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

#[cfg(feature = "ui-cli")]
#[tokio::main(flavor = "multi_thread")]
pub async fn main() -> anyhow::Result<()> {
    use backup::ui::cli::ui::BackupCli;
    use lib::ui::cli::CliUi;
    use lib::ui::Ui;

    let mut app = BackupCli::new(())?;
    app.run().await?;
    Ok(())
    // TODO :: Verify writable
    // TODO :: Verify enough space
    // TODO :: Verify dir is either empty, or has existing backup data
}
