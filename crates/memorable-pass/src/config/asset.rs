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

use anyhow::Context;
use rust_embed::RustEmbed;
use std::collections::HashMap;
use std::sync::LazyLock;
use tracing::instrument;

pub static WORDS: LazyLock<HashMap<usize, Vec<String>>> = LazyLock::new(get_words);

#[derive(RustEmbed)]
#[folder = "assets"]
pub struct Asset;

#[instrument(level = "TRACE", ret)]
fn get_words() -> HashMap<usize, Vec<String>> {
    let asset_file =
        Asset::get("words.json").context("Find words.json asset file.").expect("Failed to find words.json asset file.");
    serde_json::from_slice::<HashMap<usize, Vec<String>>>(&asset_file.data)
        .context("Parse words.json asset into Map")
        .expect("Failed to parse words.json asset into Map")
}
