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
use simplelog::{debug, error, info};
use std::collections::HashMap;
use std::sync::LazyLock;

pub(crate) static WORDS: LazyLock<HashMap<usize, Vec<String>>> = LazyLock::new(get_words);

#[derive(RustEmbed)]
#[folder = "assets"]
pub struct Asset;

fn get_words() -> HashMap<usize, Vec<String>> {
    let start = std::time::Instant::now();

    let asset_file = Asset::get("words.json").context("Find words.json asset file.").expect("Failed to find words.json asset file.");
    let words_vec = serde_json::from_slice::<Vec<Vec<String>>>(&asset_file.data)
        .context("Parse words.json asset into Vec");
    let words_map = serde_json::from_slice::<HashMap<usize, Vec<String>>>(&asset_file.data)
        .context("Parse words.json asset into Map").expect("Failed to parse words.json asset into Map");

    match words_vec {
        Ok(_) => info!("Parsed as Vec<Vec<String>>"),
        Err(e) => error!("No vec parse :( {e:?}"),
    }

    debug!("Loaded words in {}ms", start.elapsed().as_millis());

    words_map
}
