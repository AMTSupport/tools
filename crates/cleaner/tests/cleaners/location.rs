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
use assert_fs::fixture::ChildPath;
use assert_fs::{prelude::*, NamedTempFile, TempDir};
use cleaner::cleaners::location::*;
use std::borrow::Borrow;
use std::sync::LazyLock;
use std::{env, path::PathBuf};
use tracing::info;

#[test_log::test(test)]
fn test_environment_variable_exists() -> Result<()> {
    // Set up a temporary directory and a temporary environment variable
    let temp_dir = TempDir::new()?;
    let var_name = "MY_TEST_VAR";
    let var_value = temp_dir.path().to_str().unwrap();
    env::set_var(var_name, var_value);

    // Create a Location::Environment with the variable name
    let location = Location::Environment(var_name.to_string());

    // Get the path and assert that it exists and is the same as the temporary directory
    let path = location.get_path();
    assert_eq!(path.len(), 1);
    assert_eq!(path[0], PathBuf::from(var_value));

    Ok(())
}

#[test_log::test(test)]
fn test_environment_variable_nonexistent() {
    // Create a Location::Environment with a non-existent variable
    let var_name = "NON_EXISTENT_VAR";
    let location = Location::Environment(var_name.to_string());

    // Get the path and assert that it's empty
    let path = location.get_path();
    assert!(path.is_empty());
}

#[test_log::test(test)]
fn test_globbing_pattern() -> Result<()> {
    // Create a temporary directory and add some files
    let temp_dir = TempDir::new()?;
    temp_dir.child("file1.txt").touch()?;
    temp_dir.child("file2.txt").touch()?;
    temp_dir.child("otherfile.txt").touch()?;

    // Create a Location::Globbing with a glob pattern
    let pattern = format!("{}/file*.txt", temp_dir.path().to_str().unwrap());
    let location = Location::Globbing(pattern);

    // Get the path and assert that it contains the expected files
    let path = location.get_path();
    assert_eq!(path.len(), 2);
    assert_eq!(
        path,
        vec![temp_dir.child("file1.txt").path(), temp_dir.child("file2.txt").path()]
    );

    Ok(())
}

const DIR_PREFIX: &str = "test_sub_location";
static PARENT_LOCATION: LazyLock<Location> =
    LazyLock::new(|| Location::Globbing(format!("{}/.tmp*/{DIR_PREFIX}", env::temp_dir().to_str().unwrap())));
#[test_log::test(test)]
fn test_sub_location() -> Result<()> {
    // Create a temporary directory and add some files
    let temp_dir1 = NamedTempFile::new(DIR_PREFIX)?;
    let temp_dir2 = NamedTempFile::new(DIR_PREFIX)?;
    for temp_dir in vec![&temp_dir1, &temp_dir2] {
        ChildPath::new(temp_dir.join("file1.txt")).touch()?;
        ChildPath::new(temp_dir.join("file2.txt")).touch()?;
        ChildPath::new(temp_dir.join("otherfile.txt")).touch()?;
    }
    let location = Location::Sub(&PARENT_LOCATION, "file*.txt".into());

    // Get the path and assert that it contains the expected files
    let path = location.get_path();
    assert_eq!(path.len(), 4);
    assert!(path.contains(&temp_dir1.path().join("file1.txt")));
    assert!(path.contains(&temp_dir1.path().join("file2.txt")));
    assert!(path.contains(&temp_dir2.path().join("file1.txt")));
    assert!(path.contains(&temp_dir2.path().join("file2.txt")));

    Ok(())
}
