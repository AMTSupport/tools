#[cfg(windows)]
use crate::builder::CleanableBuilderTrait;
use crate::builder::{AgeType, CleanableBuilder};
use anyhow::Context;
use chrono::Duration;
use indicatif::{ParallelProgressIterator, ProgressBar, ProgressIterator};
use lib::cli::Flags;
use once_cell::sync::Lazy;
use rayon::prelude::*;
use simplelog::{debug, error, trace, warn};
use std::collections::HashSet;
use std::ops::Not;
use std::path::{Path, PathBuf};

pub mod application;
pub mod builder;

#[derive(Debug, Clone)]
pub enum PathCollections {
    Drive,
    User,
}

#[cfg(windows)]
const ENV_PROGRAMDATA: &str = "ProgramData";
#[cfg(windows)]
const ENV_WINDIR: &str = "windir";

#[cfg(windows)]
pub static LOCATIONS: Lazy<Vec<CleanableBuilder>> = Lazy::new(|| {
    vec![
        CleanableBuilder::collection(PathCollections::Drive)
            .path("$Recycle.Bin")
            .auto()
            .minimum_age(Duration::weeks(1))
            .duration_from(AgeType::FromModification),
        CleanableBuilder::env(ENV_PROGRAMDATA).path("NVIDIA").auto(),
        CleanableBuilder::env(ENV_PROGRAMDATA)
            .path("Nvidia Corporation/Downloader")
            .auto(),
        CleanableBuilder::env(ENV_PROGRAMDATA)
            .path("Microsoft/Windows/WER/ReportArchive")
            .auto(),
        CleanableBuilder::env(ENV_PROGRAMDATA)
            .path("NinitePro/NiniteDownloads/Files")
            .auto(),
        CleanableBuilder::env(ENV_WINDIR)
            .path("Downloaded Program Files")
            .auto(),
        CleanableBuilder::env(ENV_WINDIR).path("SoftwareDistribution/Download"),
        CleanableBuilder::env(ENV_WINDIR).path("Prefetch").auto(),
        CleanableBuilder::env(ENV_WINDIR).path("Temp").auto(),
        CleanableBuilder::env(ENV_WINDIR)
            .minimum_age(Duration::weeks(2))
            .path("Panther")
            .duration_from(AgeType::FromModification)
            .auto(),
        CleanableBuilder::env(ENV_WINDIR)
            .minimum_age(Duration::weeks(1))
            .path("Minidump")
            .auto(),
        CleanableBuilder::collection(PathCollections::User)
            .path("AppData/Local/Temp")
            .auto(),
        CleanableBuilder::collection(PathCollections::User)
            .path("AppData/Local/Microsoft/Windows/INetCache/IE")
            .auto(),
        CleanableBuilder::collection(PathCollections::User)
            .path("AppData/Local/Microsoft/Edge/User Data/Default/Cache")
            .auto(),
        CleanableBuilder::collection(PathCollections::User)
            .path("AppData/Local/Microsoft/Edge/User Data/Default/Code Cache")
            .auto(),
        CleanableBuilder::collection(PathCollections::User)
            .path("AppData/Local/Microsoft/Edge/User Data/Default/GPUCache")
            .auto(),
        CleanableBuilder::collection(PathCollections::User)
            .path("AppData/Local/Google/Chrome/User Data/Default/Cache")
            .auto(),
        CleanableBuilder::collection(PathCollections::User)
            .path("AppData/Local/Google/Chrome/User Data/Default/Code Cache")
            .auto(),
        CleanableBuilder::collection(PathCollections::User)
            .path("AppData/Local/Google/Chrome/User Data/Default/GPUCache")
            .auto(),
        CleanableBuilder::collection(PathCollections::User)
            .path("AppData/Local/Mozilla/Firefox/Profiles/*.default-release/cache2")
            .auto(),
        CleanableBuilder::collection(PathCollections::User)
            .path("AppData/Local/D3DSCache")
            .auto(),
        CleanableBuilder::collection(PathCollections::User)
            .path("AppData/Local/NVIDIA/DXCache")
            .auto(),
        // TODO :: Figure out how to clean these without restart explorer.exe
        CleanableBuilder::collection(PathCollections::User)
            .path("AppData/Local/Microsoft/Windows/Explorer/thumbcache_*")
            .auto(),
        CleanableBuilder::collection(PathCollections::User)
            .path("AppData/Local/Microsoft/Windows/Explorer/iconcache_*")
            .auto(),
    ]
});

#[cfg(unix)]
pub static LOCATIONS: Lazy<Vec<CleanableBuilder>> = Lazy::new(Vec::new);

pub struct CleanablePath {
    pub base_buf: String,
    pub paths: Vec<PathBuf>,
    pub auto: bool,
    pub minimum_age: Duration,
    pub duration_from: AgeType,
}

pub trait Indexed {
    fn prepare(
        &self,
        cli: &Flags,
        bar: ProgressBar,
    ) -> anyhow::Result<(PreparedPaths, PreparedPaths)>;
}

#[derive(Default, Clone, Debug)]
pub struct PreparedPaths {
    paths: Vec<PathBuf>,
    disk_size: u64,
}

impl PreparedPaths {
    fn merge_with(&mut self, other: PreparedPaths) {
        self.paths.extend(other.paths);
        self.disk_size += other.disk_size;
    }
}

impl Indexed for CleanablePath {
    fn prepare(
        &self,
        _cli: &Flags,
        bar: ProgressBar,
    ) -> anyhow::Result<(PreparedPaths, PreparedPaths)> {
        let mut prepared_auto = PreparedPaths {
            paths: Vec::new(),
            disk_size: 0,
        };
        let mut prepared_manual = PreparedPaths {
            paths: Vec::new(),
            disk_size: 0,
        };

        let bar = bar.with_message(format!("Collecting files from {}", self.base_buf));
        let iter = self.paths
            .iter()
            .flat_map(|path| path.collect())
            .progress_with(bar);

        for buf in iter {
            if self.newer_than_allowed(&buf)? {
                trace!("File is newer than allowed: {}", buf.display());
                continue;
            }

            if self.auto.not() {
                trace!("File is not auto cleanable: {}", buf.display());
                prepared_manual.disk_size += buf.metadata()?.len();
                prepared_manual.paths.push(buf);
                continue;
            }

            prepared_auto.disk_size += buf.metadata()?.len();
            prepared_auto.paths.push(buf);
        }

        Ok((prepared_auto, prepared_manual))
    }
}

impl CleanablePath {
    fn newer_than_allowed(&self, buf: &Path) -> anyhow::Result<bool> {
        // There is no minimum age set, so it's allowed to be removed.
        if self.minimum_age.is_zero() {
            return Ok(false);
        }

        let metadata = buf
            .metadata()
            .with_context(|| format!("Retrieve metadata for age check: {}", buf.display()))?;
        let now = std::time::SystemTime::now();
        let from_time = match self.duration_from {
            AgeType::FromAccess => metadata
                .accessed()
                .with_context(|| format!("Get last accessed age for {}", buf.display()))?, // TODO :: Could error on some filesystems
            AgeType::FromCreation => metadata
                .created()
                .with_context(|| format!("Get creation age for {}", buf.display()))?,
            AgeType::FromModification => metadata
                .modified()
                .with_context(|| format!("Get last modified age for {}", buf.display()))?,
        };

        let duration = now
            .duration_since(from_time)
            .with_context(|| format!("Calculate duration for {}", buf.display(),))?;

        Ok(duration.as_millis() < self.minimum_age.num_milliseconds() as u128)
    }
}

pub trait Collectable {
    fn collect(&self) -> Vec<PathBuf>;
}

impl Collectable for CleanablePath {
    fn collect(&self) -> Vec<PathBuf> {
        let mut inner_files = Vec::<PathBuf>::new();

        for path in &self.paths {
            inner_files.extend(path.collect());
        }

        inner_files
    }
}

impl Collectable for PathBuf {
    fn collect(&self) -> Vec<PathBuf> {
        let mut inner_files = Vec::<PathBuf>::new();
        let mut seen: HashSet<PathBuf> = HashSet::new();
        let mut issues = Vec::<(PathBuf, &str)>::new();
        let mut iter: Vec<PathBuf> = vec![];

        trace!("Collecting: {0}", self.display());

        let append = move |iter: &mut Vec<PathBuf>, parent: &PathBuf| match globbed(parent) {
            None => {
                trace!("No globbed files found: {0}", parent.display());
                iter.push(parent.clone());
            }
            Some(glob) => {
                trace!(
                    "Found {num} globbed files for {parent}",
                    num = glob.len(),
                    parent = parent.display()
                );

                iter.extend(glob);
            }
        };

        append(&mut iter, self);

        while let Some(path) = iter.pop() {
            if seen.get(&path).is_none() {
                seen.insert(path.clone());
            } else {
                continue;
            }

            let meta = path.metadata().unwrap();

            if meta.is_symlink() {
                trace!("Skipping symlink: {0}", &path.display());
                issues.push((path.clone(), "Symlink"));
                continue;
            }

            #[cfg(unix)]
            if permissions::is_removable(&path).is_ok_and(|x| !x) {
                trace!("Skipping non-removable: {0}", &path.display());
                issues.push((path.clone(), "Non-removable"));
            }

            #[cfg(windows)] // TODO :: Implement this for windows
            if false {
                trace!("Skipping non-removable: {0}", &path.display());
                issues.push((path.clone(), "Non-removable"));
            }

            if meta.is_dir() {
                trace!("Found directory: {0}", &path.display());
                append(&mut iter, &path);
                continue;
            }

            if meta.is_file() {
                trace!("Found file: {0}", &path.display());
                inner_files.push(path.clone());
                continue;
            }

            trace!("Issue with: {0}", &path.display());
            issues.push((path.clone(), "Unknown issue"));
        }

        if !issues.is_empty() {
            let mapped_issues = issues
                .iter()
                .map(|issue| {
                    format!(
                        "\t{1}: {0}",
                        issue.1,
                        issue.0.to_path_buf().to_str().unwrap()
                    )
                })
                .collect::<Vec<String>>()
                .join("\n");

            warn!("Issues with {0}:\n{1}", self.display(), mapped_issues);
        }

        inner_files
    }
}

pub(crate) fn globbed(buf: &PathBuf) -> Option<Vec<PathBuf>> {
    let glob_str = match buf {
        buf if buf.exists() && buf.is_dir() => format!("{0}/*", buf.to_str().unwrap()),
        buf if buf.exists() && buf.is_file() => return Some(vec![buf.clone()]),
        buf => format!("{buf}/*", buf = buf.to_str().unwrap()),
    };

    debug!("Globbing: {0}", &glob_str);

    match glob::glob(&glob_str) {
        Err(e) => {
            error!("Failed to glob: {0}", e);
            None
        }
        Ok(paths) => Some(paths.filter_map(|x| x.ok()).collect::<Vec<PathBuf>>()),
    }
}

pub(crate) fn clean(prepared: &PreparedPaths) -> anyhow::Result<u64> {
    // TODO :: If all files in directory are removable call remove_dir, else call remove_file on each.
    Ok(prepared
        .paths
        .par_iter()
        .progress_count(prepared.paths.len() as u64)
        .map(|buf| {
            match std::fs::remove_file(buf).with_context(|| format!("Delete {}", buf.display())) {
                Ok(_) => {
                    trace!("Deleted: {0}", buf.display());
                    0
                }
                Err(e) => {
                    trace!("Failed to delete: {0} ({1})", buf.display(), e);
                    buf.metadata().unwrap().len()
                }
            }
        })
        .sum())
}
