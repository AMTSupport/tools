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

// use anyhow::Result;

// #[derive(Debug, Serialize, Deserialize)]
// struct WordData {
//     accuracy: f64,
//     response_time: f64,
//     aoa_test_based: Option<f64>,
//     aoa_rating: Option<f64>,
//     concreteness: Option<f64>,
//     percent_known: Option<f64>,
//     arousal: Option<f64>,
//     dominance: Option<f64>,
//     valence: Option<f64>,
//     frequency: f64,
// }
//
// fn download_extract_zip<'a>(url: &'a str, fname: &'a str) -> Result<Vec<u8>> {
//     let response = reqwest::blocking::get(url)?;
//     let content = response.bytes()?;
//     let cursor = Cursor::new(content);
//
//     let mut zip_archive = ZipArchive::new(cursor)?;
//     let mut zip_file = zip_archive.by_name(fname)?;
//     let mut bytes = Vec::new();
//     zip_file.read_to_end(&mut bytes)?;
//
//     Ok(bytes)
// }
//
// fn get_bad_words() -> Result<HashSet<String>> {
//     let mut badwords: HashSet<String> =
//         reqwest::blocking::get("https://www.cs.cmu.edu/~biglou/resources/bad-words.txt")?
//             .text()?
//             .lines()
//             .map(|line| line.to_string())
//             .collect();
//     badwords.insert("masturbation".to_string());
//
//     Ok(badwords)
// }
//
// fn extract_words(zip_file: Vec<u8>) -> Result<Vec<String>> {
//     let mut words = Vec::new();
//     // for record in csv_reader.records() {
//     //     let record = record?;
//     //     let word = record.get(0).unwrap().to_string();
//     //     if !word.contains(" ") {
//     //         words.push(word);
//     //     }
//     // }
//
//     Ok(words)
// }

fn main() {}
// let bad_words = get_bad_words()?;
// let words = extract_words(download_extract_zip(
//     "https://www.cs.cmu.edu/~enron/enron_mail_20150507.tar.gz",
//     "enron-v1.csv",
// )?)?;
//
// let mut length_map: HashMap<usize, Vec<String>> = HashMap::new();
// for word in words.into_iter() {
//     if bad_words.contains(&word) {
//         continue;
//     }
//
//     match length_map.get_mut(&word.len()) {
//         Some(vec) => vec.push(word),
//         None => {
//             length_map.insert(word.len(), vec![word]);
//         }
//     };
// }
//
// let json_data = serde_json::to_vec(&length_map)?;
// let mut json_file = File::create("word_data.json")?;
// json_file.write_all(&*json_data)?;
