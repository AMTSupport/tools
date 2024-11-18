/*
 * Copyright (c) 2023-2024. James Draycott <me@racci.dev>
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, version 3.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.
 * See the GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License along with this program.
 * If not, see <https://www.gnu.org/licenses/>.
 */

use lib::pathed::{ensure_directory_exists, Pathed};
use std::env::temp_dir;
use std::path::PathBuf;

struct TestSource;

impl Pathed<PathBuf> for TestSource {
    const NAME: &'static str = "test_source";

    fn get_unique_name(&self) -> String {
        "unique_name".to_string()
    }
}

#[test]
fn test_base_dir() {
    let temp_dir = temp_dir();
    let result = TestSource::base_dir(&temp_dir.as_path().to_path_buf());
    assert!(result.is_ok());

    let base_dir_path = result.unwrap();
    assert!(base_dir_path.exists());
    assert!(base_dir_path.is_dir());
}

#[test]
fn test_unique_dir() {
    let temp_dir = temp_dir();
    let source = TestSource;

    let result = source.unique_dir(&temp_dir.as_path().to_path_buf());
    assert!(result.is_ok());

    let unique_dir_path = result.unwrap();
    assert!(unique_dir_path.exists());
    assert!(unique_dir_path.is_dir());
}

#[test]
fn test_ensure_directory_exists() {
    let temp_dir = temp_dir();
    let test_dir = temp_dir.as_path().join("test_dir");

    // Ensure directory doesn't exist initially
    assert!(!test_dir.exists());

    // Call ensure_directory_exists
    let result = ensure_directory_exists(&test_dir);
    assert!(result.is_ok());

    // Check that the directory now exists
    assert!(test_dir.exists());
    assert!(test_dir.is_dir());
}
