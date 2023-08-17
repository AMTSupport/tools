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

use anyhow::{Context, Result};
use chrono::Duration;
use macros::{EnumNames, EnumRegex, EnumVariants};
use readable_regex::{
    either, ends_with, everything, named_group, non_capture_group, optional, starts_and_ends_with, starts_with,
    ReadableRe,
};
use serde::{Deserialize, Serialize};
use std::cell::LazyCell;
use std::collections::HashMap;
use std::fs::Metadata;
use std::ops::Add;
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use tracing::info;

#[derive(Debug, Clone, PartialOrd, PartialEq, Serialize, Deserialize, EnumVariants, EnumNames, EnumRegex)]
pub enum Tag {
    None,
    Hourly,
    Daily,
    Weekly,
    Monthly,
    Yearly,
}

impl Tag {
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

    /// Applies the applicable tags to the file.
    /// This may be multiple tags, or none.
    pub fn tag(path: Path) -> Result<PathBuf> {
        let now = SystemTime::now();
        let mtime = path.metadata()?.modified()?;
        let age = Duration::from_std(now.duration_since(mtime)?)?;

        let mut path = path.into_path_buf();
        for tag in Self::get_variants() {
            if age < tag.duration() {
                continue;
            }

            path = tag.add_tag(path)?;
        }

        Ok(path)
    }

    pub fn add_tag(&self, path: PathBuf) -> Result<PathBuf> {
        let file_name = path.file_name().context("Getting file name")?;
        let file_name = file_name.to_str().context("Converting file name to string")?;
        let (mut tags, file_name) = Self::get_tags(file_name)?;

        if tags.contains(&self) {
            return Ok(path);
        }

        tags.push(self.clone());
        tags.sort();

        let tag = tags.iter().map(|tag| tag.name()).collect::<Vec<&str>>().join("-");
        let new_path = path.with_file_name(format!("{}-{}", tag, file_name));
        std::fs::rename(path, &new_path).context("Rename file")?;

        Ok(new_path)
    }

    pub fn remove_tag(&self, path: Path) -> Result<PathBuf> {
        let file_name = path.file_name().context("Getting file name")?;
        let file_name = file_name.to_str().context("Converting file name to string")?;
        let (mut tags, file_name) = Self::get_tags(file_name)?;

        if !tags.contains(&self) {
            return Ok(path.into_path_buf());
        }

        tags.retain(|tag| tag != self);
        tags.sort();

        let tag = tags.iter().map(|tag| tag.name()).collect::<Vec<&str>>().join("-");
        let new_path = path.with_file_name(format!("{}-{}", tag, file_name));
        std::fs::rename(path, &new_path).context("Rename file")?;

        Ok(new_path)
    }

    /// Gets the tags from the file name.
    /// This may be multiple tags, or a single length vec of None.
    /// The returned value is a tuple of the tags and the stipped file name.
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
    pub fn get_tags(str: &str) -> Result<(Vec<Tag>, &str)> {
        match starts_and_ends_with(non_capture_group(ReadableRe::String(Self::MULTI_REGEX.into())))
            .compile()
            .context("Regex Compilation Error for getting existing tags")?
            .captures(str)
        {
            None => return Ok((vec![Tag::None], str)),
            Some(captures) => Ok((
                captures.iter().map(|m| m.unwrap().as_str().into()).collect::<Vec<Tag>>(),
                str.strip_prefix(captures.get(0).unwrap().as_str()).unwrap(),
            )),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoPrune {
    /// Whether or not the auto prune feature is enabled.
    pub enabled: bool,

    /// How many hours of backups should be kept.
    pub hours: usize,

    /// How many days of backups should be kept.
    pub days: usize,

    /// How many per week backups should be kept.
    pub weeks: usize,

    /// How many per month backups should be kept.
    pub months: usize,

    /// The minimum number of backups to keep ignoring the keep_for duration.
    pub keep_latest: usize,
}

impl AutoPrune {
    const REGEX: LazyCell<ReadableRe<'static>> = LazyCell::new(|| {
        starts_with(optional(named_group(
            "tag",
            either(Tag::get_variants().iter().map(Tag::name).map(str::to_lowercase)),
        )))
        .add(ends_with(everything()))
    });

    pub fn partition_prune(&self, paths: Vec<PathBuf>) -> Result<(Vec<Path>, Vec<Path>)> {
        let now = SystemTime::now();
        let mut keep = Vec::new();
        let mut prune = Vec::new();

        let mut time_paired = paths
            .into_iter()
            .filter(|path| path.is_file())
            .filter_map(|path| path.metadata().map(|meta| (path, meta)).ok())
            .map(|(path, meta)| {
                let mtime = meta.modified().unwrap();
                let age = now.duration_since(mtime).unwrap();
                let age = Duration::from_std(age).unwrap();
                (path, meta, age)
            })
            .collect::<Vec<(Path, Metadata, Duration)>>();
        time_paired.sort_by(|(_, _, time_a), (_, _, time_b)| time_a.cmp(time_b));

        // Keep the newest files for keep_latest.
        for (path, _, _) in time_paired.iter().rev().take(self.keep_latest) {
            keep.push(path.into());
        }

        Ok((keep, prune))
    }

    pub fn should_prune(&self, file: &Path, remaining_files: usize) -> Result<bool> {
        if self.enabled == false {
            return Ok(false);
        }

        let mtime = file.metadata()?.modified()?;
        let now = SystemTime::now();
        let age = now.duration_since(mtime)?;
        let days = Duration::from_std(age)?.num_days();

        Ok(days > self.days as i64 && remaining_files > self.keep_latest)
    }

    fn auto_remove(&self, files: Vec<PathBuf>) -> Result<Vec<PathBuf>> {
        let now = SystemTime::now();
        let map = Self::tag_map(files)?;

        for tag in Tag::get_variants() {
            let (date_limit, count_limit) = match tag {
                Tag::None => continue,
                Tag::Hourly => (now - Duration::hours(self.hours as i64), self.hours),
                Tag::Daily => (now - Duration::days(self.days as i64), self.days),
                Tag::Weekly => (now - Duration::weeks(self.weeks as i64), self.weeks),
                Tag::Monthly => (now - Duration::days(self.months as i64 * 30), self.months),
                Tag::Yearly => (now - Duration::days(self.months as i64 * 365), self.months),
            };

            let files = map.get(&tag).context("Getting files for tag, should never fail")?;
            let mut file_count = files.len();

            while file_count > count_limit {
                if file_count == files.len() {
                    info!(
                        "Maximum backups exceeded for tag {}, removing oldest backups",
                        tag.name()
                    );
                }

                let file = &files[file_count - 1];
                if file.metadata()?.modified()? < date_limit {
                    tag.remove_tag(file)?;
                    file_count -= 1;

                    // Remove from map?

                    info!("Removed tag from {}", file.display());
                }
            }
        }

        Ok(map.into_iter().flat_map(|(_, files)| files).collect())
    }

    fn remove_untagged(&self, files: Vec<PathBuf>) -> Result<()> {
        let mut untagged = Vec::new();
        for file in files {
            if Self::get_tag(&file)? == Tag::None {
                untagged.push(file);
            }
        }

        for file in untagged {
            std::fs::remove_file(file)?;
        }

        Ok(())
    }

    fn time_sorted(paths: Vec<PathBuf>) -> Result<Vec<PathBuf>> {
        let mut time_paired = paths
            .into_iter()
            .filter_map(|path| path.metadata().map(|meta| (path, meta)).ok())
            .map(|(path, meta)| {
                let mtime = meta.modified().context("Getting modified time")?;
                let age = SystemTime::now().duration_since(mtime).context("Getting age")?;
                let age = Duration::from_std(age).context("Converting std::time::Duration to chrono::Duration")?;
                (path, meta, age)
            })
            .collect::<Vec<(Path, Metadata, Duration)>>();
        time_paired.sort_by(|(_, _, time_a), (_, _, time_b)| time_a.cmp(time_b));

        Ok(time_paired.into_iter().map(|(path, _, _)| path).collect())
    }

    fn tag_map(paths: Vec<PathBuf>) -> Result<HashMap<Tag, Vec<PathBuf>>> {
        let mut tags = HashMap::new();
        for path in paths {
            let tag = Self::get_tag(&path)?;
            tags.entry(tag).or_insert_with(Vec::new).push(path);
        }

        for (tag, paths) in tags {
            tags.insert(tag, Self::time_sorted(paths)?);
        }

        Ok(tags)
    }
}

impl Default for AutoPrune {
    fn default() -> Self {
        Self {
            enabled: false,
            hours: 0,
            days: 14,
            weeks: 0,
            months: 0,
            keep_latest: 5,
        }
    }
}
