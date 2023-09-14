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

#[cfg(not(any(feature = "ui-gui", feature = "ui-tui", feature = "ui-cli")))]
fn main() {
    use std::process::exit;

    compile_error!("No UI selected, please select one of the following features: ui-gui, ui-tui, ui-cli");
}

#[cfg(any(feature = "ui-gui", feature = "ui-tui", feature = "ui-cli"))]
fn main() {}
