/*
 * Copyright (c) 2024. James Draycott <me@racci.dev>
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

use crate::config::rules::metadata::Metadata;
use crate::config::rules::rule::Rule;
use anyhow::{Context, Result};
use chrono::{DateTime, Duration, Utc};
use fs_err as fs;
use lib::builder;
use macros::{EnumNames, EnumRegex, EnumVariants};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::ffi::OsStr;
use std::fmt::Debug;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::time::SystemTime;
use tokio_stream::StreamExt;
use tracing::{debug, error, info, instrument, trace, warn};

#[derive(
    Default,
    Debug,
    Clone,
    Copy,
    Hash,
    PartialOrd,
    Ord,
    PartialEq,
    Eq,
    Serialize,
    Deserialize,
    EnumVariants,
    EnumNames,
    EnumRegex,
)]
pub enum Tag {
    #[default]
    None,
    Hourly,
    Daily,
    Weekly,
    Monthly,
    Yearly,
}

impl Tag {
    /// Gets the max duration that this tag covers.
    pub fn duration(&self) -> Duration {
        match self {
            Self::None => Duration::zero(),
            Self::Hourly => Duration::hours(1),
            Self::Daily => Duration::days(1),
            Self::Weekly => Duration::weeks(1),
            Self::Monthly => Duration::days(30),
            Self::Yearly => Duration::days(365),
        }
    }

    /// Tests if the tag is applicable to the supplied [`Metadata`]
    ///
    /// This tests the files modified time against the max duration of the tag.
    #[inline]
    pub fn applicable(&self, metadata: &Metadata) -> bool {
        let now = Utc::now();
        let age = now.signed_duration_since(metadata.mtime);

        age < self.duration()
    }

    /// Applies the applicable tags to the file.
    /// This may be multiple tags, or none.
    #[instrument(level = "TRACE")]
    pub fn tag(path: &Path) -> PathBuf {
        let mut path = path.to_path_buf();
        let metadata = path.metadata().unwrap();
        let metadata = Metadata::from(metadata);
        for tag in Self::applicable_tags(&metadata) {
            path = tag.add_tag(&path);
        }

        path
    }

    pub fn applicable_tags(metadata: &Metadata) -> Vec<Tag> {
        let mut tags = Vec::new();
        for tag in Tag::get_variants() {
            if !tag.applicable(metadata) {
                continue;
            }

            tags.push(tag);
        }

        tags
    }

    /// Appends the tag to the file name.
    /// If this tag is already present there is no change.
    /// If there are other tags present they will be sorted.
    #[instrument(level = "TRACE")]
    pub fn add_tag(&self, path: &Path) -> PathBuf {
        let file_name = match Self::get_file_name_or_ret(path) {
            Ok(str) => str,
            Err(pb) => return pb,
        };
        let (mut tags, file_name) = Self::get_tags(file_name);

        if tags.contains(&Tag::None) {
            tags.retain(|tag| tag != &Tag::None);
        }

        if tags.contains(self) {
            warn!(
                "Tag {} already present in file name {}, this shouldn't happen.",
                self.name(),
                file_name
            );
            return path.to_path_buf();
        }

        tags.push(*self);
        tags.sort();

        let tag = tags.iter().map(|tag| tag.name()).collect::<Vec<&str>>().join("-");
        let new_path = path.with_file_name(format!("{}-{}", tag, file_name));
        fs::rename(path, &new_path).unwrap();

        new_path
    }

    #[instrument(level = "TRACE")]
    pub fn remove_tag(&self, path: &Path) -> PathBuf {
        let file_name = match Self::get_file_name_or_ret(path) {
            Ok(str) => str,
            Err(pb) => return pb,
        };
        let (mut tags, file_name) = Self::get_tags(file_name);

        if !tags.contains(self) {
            return path.to_path_buf();
        }

        tags.retain(|tag| tag != self);
        tags.sort();

        let tag = tags.iter().map(|tag| tag.name()).collect::<Vec<&str>>().join("-");
        let new_path = path.with_file_name(format!("{}-{}", tag, file_name));
        match fs::rename(path, &new_path) {
            Ok(_) => trace!("Renamed file from {} to {}", path.display(), new_path.display()),
            Err(err) => error!(
                "Error while renaming file from {} to {}: {err}",
                path.display(),
                new_path.display()
            ),
        }

        new_path
    }

    /// Gets the tags from the file name.
    ///
    /// This may be multiple tags, or a single length vec of None.
    /// The returned value is a tuple of the tags and the stripped file name.
    ///
    /// # Example
    /// ```
    /// use backup::config::rules::autoprune::{AutoPrune, Tag};
    /// use anyhow::Result;
    /// use std::path::Path;
    ///
    /// let (tags, file_name) = Tag::get_tags("hourly-daily-weekly-monthly-yearly-file.txt")?;
    /// assert_eq!(tags, vec![Tag::Hourly, Tag::Daily, Tag::Weekly, Tag::Monthly, Tag::Yearly]);
    /// assert_eq!(file_name, "file.txt");
    ///
    /// let (tags, file_name) = Tag::get_tags("file.txt")?;
    /// assert_eq!(tags, vec![Tag::None]);
    /// assert_eq!(file_name, "file.txt");
    /// ```
    #[instrument(level = "TRACE")]
    pub fn get_tags(str: &str) -> (Vec<Tag>, &str) {
        let compiled = Regex::new(Self::REGEX).expect("Regex Compilation Error for getting existing tags"); // Infallible

        debug!("Compiled regex for getting existing tags [{compiled}]");
        let capture = compiled.captures_iter(str);
        let tags = capture.map(|m| m.get(0).unwrap().as_str().parse().unwrap()).collect::<Vec<Tag>>();
        if tags.is_empty() {
            debug!("No tags found in file name [{str}]");
            return (vec![Tag::None], str);
        }

        let tag_prefix = tags.iter().fold(String::new(), |combining, next| {
            let mut str = combining.clone();
            if !combining.is_empty() {
                str.push('-');
            }

            str.push_str(next.name());
            str
        });

        let str = str.strip_prefix(&*format!("{tag_prefix}-")).unwrap();
        debug!("Returning tags [{tags:?}] and file name [{str}]");

        (tags, str)
    }

    fn get_file_name_or_ret(path: &Path) -> Result<&str, PathBuf> {
        match path.file_name().and_then(OsStr::to_str) {
            None => {
                error!("Unable to get file name from path {}", path.display());
                Err(path.to_path_buf())
            }
            Some(str) => Ok(str),
        }
    }
}

builder!(AutoPrune = [
    /// How many hours of backups should be kept.
    hours => usize = 12,
    /// How many days of backups should be kept.
    days => usize = 7,
    /// How many per week backups should be kept.
    weeks => usize = 2,
    /// How many per month backups should be kept.
    months => usize = 1,
    /// The minimum number of backups to keep ignoring the keep_for duration.
    keep_latest => usize = 5
]);

impl FromStr for AutoPrune {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut autoprune = AutoPrune { ..Default::default() };

        let mut split = s.split_whitespace();
        if let Some(hours) = split.next() {
            autoprune.hours = usize::from_str(hours)?;
        }

        if let Some(days) = split.next() {
            autoprune.days = usize::from_str(days)?;
        }

        if let Some(weeks) = split.next() {
            autoprune.weeks = usize::from_str(weeks)?;
        }

        if let Some(months) = split.next() {
            autoprune.months = usize::from_str(months)?;
        }

        if let Some(keep_latest) = split.next() {
            autoprune.keep_latest = usize::from_str(keep_latest)?;
        }

        Ok(autoprune)
    }
}

// #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
// pub struct AutoPrune {
//     /// How many hours of backups should be kept.
//     pub hours: usize,
//
//     /// How many days of backups should be kept.
//     pub days: usize,
//
//     /// How many per week backups should be kept.
//     pub weeks: usize,
//
//     /// How many per month backups should be kept.
//     pub months: usize,
//
//     /// The minimum number of backups to keep ignoring the keep_for duration.
//     pub keep_latest: usize,
// }

impl AutoPrune {
    /// This will iterate over the files, removing the tags from the oldest
    /// files until the maximum number of backups for its tag is reached.
    #[instrument(level = "TRACE")]
    pub async fn auto_remove(&self, files: Vec<PathBuf>) -> Vec<PathBuf> {
        let now = Utc::now();
        let mut map = Self::tag_map(files).await;

        debug!("Pruning tags for files from {map:?}");

        for tag in Tag::get_variants() {
            let (date_limit, count_limit) = match tag {
                Tag::None => continue,
                Tag::Hourly => (now - Duration::hours(self.hours as i64), self.hours),
                Tag::Daily => (now - Duration::days(self.days as i64), self.days),
                Tag::Weekly => (now - Duration::weeks(self.weeks as i64), self.weeks),
                Tag::Monthly => (now - Duration::days(self.months as i64 * 30), self.months),
                Tag::Yearly => (now - Duration::days(self.months as i64 * 365), self.months),
            };

            let files = map.get_mut(&tag).unwrap();
            let mut file_count = files.len();

            while file_count > count_limit {
                if file_count == files.len() {
                    info!(
                        "Maximum backups exceeded for tag {}, removing oldest backups",
                        tag.name()
                    );
                }

                let file = &*files[file_count - 1];
                // TODO : Handle gracefully
                let file_mtime = file.metadata().unwrap().modified().unwrap();
                let file_mtime = DateTime::<Utc>::from(file_mtime);
                if file_mtime < date_limit {
                    files[file_count - 1] = tag.remove_tag(file);
                    file_count -= 1;
                }
            }
        }

        map.into_iter().flat_map(|(_, files)| files).collect()
    }

    /// Remove all files that have no tags.
    ///
    /// This will delete all files which have no tags, or only the tag None.
    /// The files will be deleted permanently, there is no undo.
    ///
    /// The return value is [`Vec<&Path>`] of the files that were removed from the disk.
    #[instrument(skip(self))]
    pub async fn remove_untagged<'a, I: Iterator<Item = &'a Path> + Debug>(&'a self, files: I) -> Vec<&'a Path> {
        let mut stream = tokio_stream::iter(files).filter(|file| {
            let name = file.file_name().expect("Getting file name").to_string_lossy();
            let (tags, _) = Tag::get_tags(&name);
            matches!(tags.as_slice(), &[Tag::None])
        });

        let mut removed = Vec::new();
        while let Some(file) = stream.next().await {
            match fs::remove_file(file) {
                Ok(_) => {
                    trace!("Removed untagged file {}", file.display());
                    removed.push(file)
                }
                Err(err) => error!("Error while removing untagged file {}: {err:#}", file.display()),
            }
        }

        removed
    }

    #[instrument(level = "TRACE")]
    fn time_sorted(paths: Vec<PathBuf>) -> Vec<PathBuf> {
        let mut time_paired = paths
            .into_iter()
            .filter_map(|path| path.metadata().map(|meta| (path, meta)).ok())
            .map(|(path, meta)| {
                let mtime = meta.modified().context("Getting modified time").unwrap();
                let age = SystemTime::now().duration_since(mtime).context("Getting age").unwrap();
                let age =
                    Duration::from_std(age).context("Converting std::time::Duration to chrono::Duration").unwrap();
                (path, age)
            })
            .collect::<Vec<(PathBuf, Duration)>>();
        time_paired.sort_by(|(_, time_a), (_, time_b)| time_a.cmp(time_b));

        time_paired.into_iter().map(|(path, _)| path).collect()
    }

    #[instrument(level = "TRACE")]
    async fn tag_map(paths: Vec<PathBuf>) -> HashMap<Tag, Vec<PathBuf>> {
        let tuple_tags = Tag::get_variants().into_iter().map(|tag| (tag, Vec::new()));
        let mut map = HashMap::from_iter(tuple_tags);

        let mut stream = tokio_stream::iter(paths).map(|path| {
            let name = path.file_name().context("Getting file name").unwrap().to_string_lossy();
            let (tags, _) = Tag::get_tags(&name);
            (tags, path.clone())
        });

        while let Some((tags, path)) = stream.next().await {
            for tag in tags {
                let vec = map.get_mut(&tag).unwrap();
                vec.push(path.clone());
            }
        }

        for (key, paths) in map.clone().into_iter() {
            let sorted = Self::time_sorted(paths.clone());
            map.insert(key, sorted);
        }

        map
    }
}

impl Default for AutoPrune {
    fn default() -> Self {
        Self {
            hours: 0,
            days: 14,
            weeks: 0,
            months: 0,
            keep_latest: 5,
        }
    }
}

impl Rule for AutoPrune {
    async fn would_keep(&self, existing_files: &[&Path], _new_path: &Path, new_metadata: &Metadata) -> bool {
        if existing_files.len() < self.keep_latest {
            return true;
        }

        if Tag::applicable_tags(new_metadata) == vec![Tag::None] {
            return false;
        }

        true
    }
}
