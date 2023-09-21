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

use assert_fs::prelude::{FileTouch, PathChild};
use assert_fs::TempDir;
use backup::config::rules::autoprune::Tag;
use backup::config::rules::metadata::Metadata;
use chrono::{Duration, Utc};
use std::assert_matches::assert_matches;

// Helper function to create a temporary directory and return its path
fn create_temp_dir() -> TempDir {
    let temp_dir = TempDir::new().expect("Failed to create temporary directory");
    temp_dir
}

/// Test for Tag Durations
///
/// This test function verifies the correctness of the duration associated with each `Tag` variant.
/// The `Tag` enum represents different time intervals, and this test ensures that the
/// `duration` method of each `Tag` variant returns the expected time duration.
///
/// The following assertions are made in this test:
///
/// - `Tag::None.duration()` should return a zero duration.
/// - `Tag::Hourly.duration()` should return a duration of one hour.
/// - `Tag::Daily.duration()` should return a duration of one day.
/// - `Tag::Weekly.duration()` should return a duration of one week.
/// - `Tag::Monthly.duration()` should return a duration of approximately 30 days.
/// - `Tag::Yearly.duration()` should return a duration of approximately 365 days.
///
/// If any of these assertions fail, it indicates a problem with the `duration` method
/// implementation for the `Tag` enum.
///
/// # Examples
///
/// ```
/// use your_crate_name_here::Tag;
/// use chrono::Duration;
///
/// assert_eq!(Tag::None.duration(), Duration::zero());
/// assert_eq!(Tag::Hourly.duration(), Duration::hours(1));
/// assert_eq!(Tag::Daily.duration(), Duration::days(1));
/// assert_eq!(Tag::Weekly.duration(), Duration::weeks(1));
/// assert_eq!(Tag::Monthly.duration(), Duration::days(30));
/// assert_eq!(Tag::Yearly.duration(), Duration::days(365));
/// ```
#[test]
fn tag_duration() {
    assert_eq!(Tag::None.duration(), Duration::zero());
    assert_eq!(Tag::Hourly.duration(), Duration::hours(1));
    assert_eq!(Tag::Daily.duration(), Duration::days(1));
    assert_eq!(Tag::Weekly.duration(), Duration::weeks(1));
    assert_eq!(Tag::Monthly.duration(), Duration::days(30));
    assert_eq!(Tag::Yearly.duration(), Duration::days(365));
}

#[test]
fn tag_tag() {
    let temp_dir = create_temp_dir();
    let file = temp_dir.child("test.txt");
    file.touch().expect("Failed to create file");

    let file = Tag::tag(file.path());
    assert_matches!(file.file_name(), Some(name) if name == "Hourly-test.txt");
}

#[test_log::test(test)]
fn tag_applicable_and_applicable_tags() {
    let meta = Metadata {
        mtime: Utc::now() - Duration::minutes(5),
        size: 0,
        is_dir: false,
        is_file: true,
    };

    let applicable = Tag::applicable_tags(&meta);
    assert_matches!(
        applicable[..],
        [Tag::Hourly, Tag::Daily, Tag::Weekly, Tag::Monthly, Tag::Yearly]
    );
    assert!(Tag::Hourly.applicable(&meta));
    assert!(Tag::Daily.applicable(&meta));
    assert!(Tag::Weekly.applicable(&meta));
    assert!(Tag::Monthly.applicable(&meta));
    assert!(Tag::Yearly.applicable(&meta));

    let meta = Metadata {
        mtime: Utc::now() - Duration::hours(2),
        ..meta
    };
    let applicable = Tag::applicable_tags(&meta);
    assert_matches!(applicable[..], [Tag::Daily, Tag::Weekly, Tag::Monthly, Tag::Yearly]);
    assert!(!Tag::Hourly.applicable(&meta));
    assert!(Tag::Daily.applicable(&meta));
    assert!(Tag::Weekly.applicable(&meta));
    assert!(Tag::Monthly.applicable(&meta));
    assert!(Tag::Yearly.applicable(&meta));

    let meta = Metadata {
        mtime: Utc::now() - Duration::days(2),
        ..meta
    };
    let applicable = Tag::applicable_tags(&meta);
    assert_matches!(applicable[..], [Tag::Weekly, Tag::Monthly, Tag::Yearly]);
    assert!(!Tag::Hourly.applicable(&meta));
    assert!(!Tag::Daily.applicable(&meta));
    assert!(Tag::Weekly.applicable(&meta));
    assert!(Tag::Monthly.applicable(&meta));
    assert!(Tag::Yearly.applicable(&meta));

    let meta = Metadata {
        mtime: Utc::now() - Duration::weeks(2),
        ..meta
    };
    let applicable = Tag::applicable_tags(&meta);
    assert_matches!(applicable[..], [Tag::Monthly, Tag::Yearly]);
    assert!(!Tag::Hourly.applicable(&meta));
    assert!(!Tag::Daily.applicable(&meta));
    assert!(!Tag::Weekly.applicable(&meta));
    assert!(Tag::Monthly.applicable(&meta));
    assert!(Tag::Yearly.applicable(&meta));

    let meta = Metadata {
        mtime: Utc::now() - Duration::days(60),
        ..meta
    };
    let applicable = Tag::applicable_tags(&meta);
    assert_matches!(applicable[..], [Tag::Yearly]);
    assert!(!Tag::Hourly.applicable(&meta));
    assert!(!Tag::Daily.applicable(&meta));
    assert!(!Tag::Weekly.applicable(&meta));
    assert!(!Tag::Monthly.applicable(&meta));
    assert!(Tag::Yearly.applicable(&meta));

    let meta = Metadata {
        mtime: Utc::now() - Duration::days(400),
        ..meta
    };
    let applicable = Tag::applicable_tags(&meta);
    assert_matches!(applicable[..], []);
    assert!(!Tag::Hourly.applicable(&meta));
    assert!(!Tag::Daily.applicable(&meta));
    assert!(!Tag::Weekly.applicable(&meta));
    assert!(!Tag::Monthly.applicable(&meta));
    assert!(!Tag::Yearly.applicable(&meta));
}

#[test_log::test(test)]
fn tag_add_tag() {
    let temp_dir = create_temp_dir();
    let file = temp_dir.child("test.txt");
    file.touch().expect("Failed to create file");
    let file = Tag::Hourly.add_tag(file.path());

    assert_matches!(file.file_name(), Some(name) if name == "Hourly-test.txt", "File should have tag name prefixed.");
    assert!(file.exists(), "File should have being moved");
}

#[test_log::test(test)]
fn tag_remove_tag() {
    let temp_dir = create_temp_dir();
    let file = temp_dir.child("Hourly-test.txt");
    file.touch().expect("Failed to create file");
    let file = Tag::Hourly.remove_tag(file.path());

    assert_matches!(file.file_name(), Some(name) if name == "test.txt", "File should have tag name removed.");
    assert!(file.exists(), "File should have being moved");
}

#[test_log::test(test)]
fn tag_get_tags() {
    let temp_dir = create_temp_dir();
    let get_tags = |tags: &[Tag]| {
        let file = temp_dir.child("test.txt");
        file.touch().expect("Failed to create file");
        let mut path = file.to_path_buf();
        for tag in tags {
            path = tag.add_tag(&path);
        }

        let name = path.file_name().unwrap().to_str().unwrap();
        let (gotten_tags, _) = Tag::get_tags(name);

        assert_eq!(&gotten_tags[..], tags, "Tags should be equal for file: {name}");
    };

    get_tags(&[Tag::Hourly]);
    get_tags(&[Tag::Hourly, Tag::Daily]);
    get_tags(&[Tag::Hourly, Tag::Daily, Tag::Weekly]);
    get_tags(&[Tag::Hourly, Tag::Daily, Tag::Weekly, Tag::Monthly]);
    get_tags(&[Tag::Hourly, Tag::Daily, Tag::Weekly, Tag::Monthly, Tag::Yearly]);
}

// #[test_log::test(tokio::test)]
// async fn empty_dir() {
//     let temp_dir = create_temp_dir();
//     // Create some files in the temporary directory
//
//     // Initialize an AutoPrune instance with your desired configuration
//     let auto_prune = AutoPrune {
//         hours: 2,
//         days: 1,
//         weeks: 1,
//         months: 1,
//         keep_latest: 3,
//     };
//
//     // Call the auto_remove method with a list of files
//     let files = fs::read_dir(&*temp_dir)
//         .expect("Failed to read directory")
//         .map(|entry| entry.expect("Failed to read entry").path())
//         .collect::<Vec<PathBuf>>();
//     let removed = auto_prune.auto_remove(files).await;
//
//     assert_eq!(removed.len(), 0);
// }
//
// #[test_log::test(tokio::test)]
// async fn remove_oldest_of_tag() {
//     let temp_dir = create_temp_dir();
//     let old = temp_dir.child("oldest_tagged.txt");
//     let young = temp_dir.child("young_tagged.txt");
//     for file in [&old, &young] {
//         file.touch().expect("Failed to create file");
//         Tag::Hourly.add_tag(file);
//     }
//
//     let auto_prune = AutoPrune {
//         hours: 1,
//         days: 0,
//         weeks: 0,
//         months: 0,
//         keep_latest: 0,
//     };
//
//     let files = fs::read_dir(&*temp_dir)
//         .expect("Failed to read directory")
//         .map(|entry| entry.expect("Failed to read entry").path())
//         .collect::<Vec<PathBuf>>();
//     let result = auto_prune.auto_remove(files).await;
//
//     assert_eq!(result.len(), 1);
//     assert_eq!(result[0], old.path());
// }
