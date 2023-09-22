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

use indicatif::{ProgressBar, ProgressStyle};
use std::time::Duration;

pub const PROGRESS_CHARS: &str = "█▓▒░  ";
pub const TICK_CHARS: &str = "⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏";
pub const SPINNER_TEMPLATE: &str = "{spinner:.green} {msg:.cyan/blue}";
pub const BAR_TEMPLATE: &str = "{spinner:.green} {msg:.cyan/blue} {pos}/{len} {bar:.cyan/blue}";
pub const DOWNLOAD_TEMPLATE: &str =
    "{spinner:.green} {bytes:.cyan/blue}/{total_bytes:.cyan/blue} ({bytes_per_sec:.cyan/blue}, {eta:.cyan/blue}) {wide_bar:.cyan/blue}";

pub fn spinner() -> ProgressBar {
    let spinner = ProgressBar::new_spinner().with_style(style_spinner());
    spinner.enable_steady_tick(Duration::from_millis(50));
    spinner
}

pub fn spinner_with_count() -> ProgressBar {
    let spinner = ProgressBar::new_spinner()
        .with_style(style_spinner().template("{spinner:.green} {msg:.cyan/blue} {pos}/{len}").unwrap());
    spinner.enable_steady_tick(Duration::from_millis(50));
    spinner
}

pub fn bar(len: u64) -> ProgressBar {
    let bar = ProgressBar::new(len).with_style(style_bar());
    bar.enable_steady_tick(Duration::from_millis(50));
    bar
}

pub fn download() -> ProgressBar {
    let bar = ProgressBar::new(0).with_style(download_style());
    bar.enable_steady_tick(Duration::from_millis(50));
    bar
}

pub fn style_spinner() -> ProgressStyle {
    ProgressStyle::default_spinner()
        .tick_chars(TICK_CHARS)
        .progress_chars(PROGRESS_CHARS)
        .template(SPINNER_TEMPLATE)
        .unwrap()
}

pub fn style_bar() -> ProgressStyle {
    ProgressStyle::default_bar().tick_chars(TICK_CHARS).progress_chars(PROGRESS_CHARS).template(BAR_TEMPLATE).unwrap()
}

pub fn download_style() -> ProgressStyle {
    ProgressStyle::default_bar()
        .tick_chars(TICK_CHARS)
        .progress_chars(PROGRESS_CHARS)
        .template(DOWNLOAD_TEMPLATE)
        .unwrap()
}
