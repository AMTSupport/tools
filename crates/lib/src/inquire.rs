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

use inquire::ui::{Color, RenderConfig, StyleSheet, Styled};

pub fn inquire_style() -> RenderConfig<'static> {
    RenderConfig::default_colored()
        .with_selected_option(Some(StyleSheet::empty().with_fg(Color::White)))
        .with_highlighted_option_prefix(Styled::new("-> ").with_fg(Color::LightCyan))
}
