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

use crate::config::rules::rule::Rule;
use anyhow::{Context, Result};
use chrono::{DateTime, Duration, Utc};
use macros::{EnumNames, EnumRegex, EnumVariants};
use readable_regex::{
    ends_with, everything, non_capture_group, optional, starts_and_ends_with, starts_with, ReadableRe,
};
use serde::{Deserialize, Serialize};
use std::cell::LazyCell;
use std::collections::HashMap;
use std::ops::Add;
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use tokio_stream::StreamExt;
use tracing::{info, instrument};

#[derive(
    Debug, Clone, Copy, Hash, PartialOrd, Ord, PartialEq, Eq, Serialize, Deserialize, EnumVariants, EnumNames, EnumRegex,
)]
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
    #[instrument]
    pub fn tag(path: &Path) -> Result<PathBuf> {
        let mut path = path.to_path_buf();
        let metadata = path.metadata().context("Getting metadata")?;
        let metadata = crate::config::rules::metadata::Metadata::from(metadata);
        for tag in Self::applicable_tags(&metadata) {
            path = tag.add_tag(&path)?;
        }

        Ok(path)
    }

    #[instrument]
    pub fn applicable_tags(metadata: &crate::config::rules::metadata::Metadata) -> Vec<Tag> {
        let now = Utc::now();
        let age = now.signed_duration_since(metadata.mtime);

        let mut tags = vec![];
        for tag in Self::get_variants() {
            if age < tag.duration() {
                continue;
            }

            tags.push(tag);
        }

        tags
    }

    /// Appends the tag to the file name.
    /// If this tag is already present there is no change.
    /// If there are other tags present they will be sorted.
    #[instrument]
    pub fn add_tag(&self, path: &Path) -> Result<PathBuf> {
        let file_name = path.file_name().context("Getting file name")?;
        let file_name = file_name.to_str().context("Converting file name to string")?;
        let (mut tags, file_name) = Self::get_tags(file_name)?;

        if tags.contains(&self) {
            return Ok(path.to_path_buf());
        }

        tags.push(self.clone());
        tags.sort();

        let tag = tags.iter().map(|tag| tag.name()).collect::<Vec<&str>>().join("-");
        let new_path = path.with_file_name(format!("{}-{}", tag, file_name));
        std::fs::rename(path, &new_path).context("Rename file")?;

        Ok(new_path)
    }

    #[instrument]
    pub fn remove_tag(&self, path: &Path) -> Result<PathBuf> {
        let file_name = path.file_name().context("Getting file name")?;
        let file_name = file_name.to_str().context("Converting file name to string")?;
        let (mut tags, file_name) = Self::get_tags(file_name)?;

        if !tags.contains(&self) {
            return Ok(path.to_path_buf());
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
    #[instrument]
    pub fn get_tags(str: &str) -> Result<(Vec<Tag>, &str)> {
        match starts_and_ends_with(non_capture_group(ReadableRe::String(Self::MULTI_REGEX.into())))
            .compile()
            .context("Regex Compilation Error for getting existing tags")?
            .captures(str)
        {
            None => return Ok((vec![Tag::None], str)),
            Some(captures) => Ok((
                captures.iter().map(|m| m.unwrap().as_str().try_into().unwrap()).collect::<Vec<Tag>>(),
                str.strip_prefix(captures.get(0).unwrap().as_str()).unwrap(),
            )),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoPrune {
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
    const REGEX: LazyCell<ReadableRe<'static>> =
        LazyCell::new(|| starts_with(optional(ReadableRe::Raw(Tag::MULTI_REGEX))).add(ends_with(everything())));

    /// This will iterate over the files, removing the tags from the oldest
    /// files until the maximum number of backups for its tag is reached.
    #[instrument]
    async fn auto_remove(&self, files: Vec<PathBuf>) -> Result<Vec<PathBuf>> {
        let now = Utc::now();
        let mut map = Self::tag_map(files).await?;

        for tag in Tag::get_variants() {
            let (date_limit, count_limit) = match tag {
                Tag::None => continue,
                Tag::Hourly => (now - Duration::hours(self.hours as i64), self.hours),
                Tag::Daily => (now - Duration::days(self.days as i64), self.days),
                Tag::Weekly => (now - Duration::weeks(self.weeks as i64), self.weeks),
                Tag::Monthly => (now - Duration::days(self.months as i64 * 30), self.months),
                Tag::Yearly => (now - Duration::days(self.months as i64 * 365), self.months),
            };

            let files = map.get_mut(&tag).context("Getting files for tag, should never fail")?;
            let mut file_count = files.len();

            while file_count > count_limit {
                if file_count == files.len() {
                    info!(
                        "Maximum backups exceeded for tag {}, removing oldest backups",
                        tag.name()
                    );
                }

                let file = &*files[file_count - 1];
                let file_mtime = file.metadata()?.modified()?;
                let file_mtime = DateTime::<Utc>::from(file_mtime);
                if file_mtime < date_limit {
                    files[file_count - 1] = tag.remove_tag(file)?;
                    file_count -= 1;
                }
            }
        }

        Ok(map.into_iter().flat_map(|(_, files)| files).collect())
    }

    #[instrument]
    async fn remove_untagged(&self, files: Vec<PathBuf>) -> Result<()> {
        let mut stream = tokio_stream::iter(files).filter(|file| {
            let name = file.file_name().expect("Getting file name").to_string_lossy();
            let (tags, _) = Tag::get_tags(&name).expect("Getting tags from file name");
            tags == vec![Tag::None]
        });

        while let Some(file) = stream.next().await {
            tokio::fs::remove_file(file).await.context("Removing untagged file")?;
        }

        Ok(())
    }

    #[instrument]
    fn time_sorted(paths: Vec<PathBuf>) -> Result<Vec<PathBuf>> {
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

        Ok(time_paired.into_iter().map(|(path, _)| path).collect())
    }

    #[instrument]
    async fn tag_map(paths: Vec<PathBuf>) -> Result<HashMap<Tag, Vec<PathBuf>>> {
        let tuple_tags = Tag::get_variants().into_iter().map(|tag| (tag, Vec::new()));
        let mut map = HashMap::from_iter(tuple_tags);

        let mut stream = tokio_stream::iter(paths).map(|path| {
            let name = path.file_name().context("Getting file name")?.to_string_lossy();
            let tags = Tag::get_tags(&name);
            tags.map(|(tags, _)| (tags, path.clone()))
        });

        while let Some(Ok((tags, path))) = stream.next().await {
            for tag in tags {
                let vec = map.get_mut(&tag).context("Getting tag from map")?;
                vec.push(path.clone());
            }
        }

        for (key, paths) in map.clone().into_iter() {
            let sorted = Self::time_sorted(paths.clone())?;
            map.insert(key, sorted);
        }

        Ok(map)
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
    async fn would_keep(
        &self,
        existing_files: &[&Path],
        _new_path: &Path,
        new_metadata: &crate::config::rules::metadata::Metadata,
    ) -> bool {
        if existing_files.len() < self.keep_latest {
            return true;
        }

        if Tag::applicable_tags(&new_metadata) == vec![Tag::None] {
            return false;
        }

        true
    }
}
