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

use reqwest::StatusCode;
use thiserror::Error;
use tracing::error;

type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
enum Error {
    #[error("Failed to make {} request to {}", method, url)]
    Request {
        method: &'static str,
        url: String,
        #[source]
        source: reqwest::Error,
    },

    #[error("Failed to download zip file, status code: {0}")]
    Download(StatusCode),

    #[error("Failed to extract zip file")]
    Extract(#[from] zip::result::ZipError),
}

// TODO :: Generate assets/words.json and embed it
fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    // let bad_words = "";
    // let reader = download_extract_zip("http://crr.ugent.be/blp/txt/blp-items.txt.zip", "blp-items.txt")?;
}

// fn download_extract_zip(url: &str, filename: &str) -> Result<impl Read> {
//     let mut resp = reqwest::blocking::get(url)
//         .map_err(|err| Error::Request {
//             method: "GET",
//             url: url.to_string(),
//             source: err,
//         })?
//         .error_for_status()
//         .map_err(|err| Error::Request {
//             method: "GET",
//             url: url.to_string(),
//             source: err,
//         })?;
//
//     let mut out = fs::File::create(filename).unwrap();
//     zip::read::ZipArchive::new(resp).unwrap();
//
//     let mut out = fs::File::create(filename)?;
//     resp.copy_to(&mut out)?;
//     let file = fs::File::open(filename)?;
//     let mut archive = zip::ZipArchive::new(file)?;
//     let mut file = archive.by_index(0)?;
//     let mut out = File::create(filename)?;
//     std::io::copy(&mut file, &mut out)?;
//     let file = File::open(filename)?;
//     Ok(file)
// }
