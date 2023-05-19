#[cfg(windows)]
use crate::builder::CleanableBuilderTrait;
use crate::builder::{AgeType, CleanableBuilder};
use anyhow::Context;
use async_trait::async_trait;
use chrono::Duration;
use lib::cli::Flags;
use log::{error, info, trace, warn};
use once_cell::sync::Lazy;
use std::collections::HashSet;
use std::fs;
use std::ops::Not;
use std::path::PathBuf;

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
            .path(r"$Recycle.Bin")
            .auto()
            .minimum_age(Duration::weeks(1))
            .duration_from(AgeType::FromModification),
        CleanableBuilder::env(ENV_PROGRAMDATA).path("NVIDIA").auto(),
        CleanableBuilder::env(ENV_PROGRAMDATA)
            .path(r"Nvidia Corporation\Downloader")
            .auto(),
        CleanableBuilder::env(ENV_PROGRAMDATA)
            .path(r"Microsoft\Windows\WER\ReportArchive")
            .auto(),
        CleanableBuilder::env(ENV_PROGRAMDATA)
            .path(r"NinitePro\NiniteDownloads\Files")
            .auto(),
        CleanableBuilder::env(ENV_WINDIR)
            .path("Downloaded Program Files")
            .auto(),
        CleanableBuilder::env(ENV_WINDIR).path(r"SoftwareDistribution\Download"),
        CleanableBuilder::env(ENV_WINDIR).path("Prefetch").auto(),
        CleanableBuilder::env(ENV_WINDIR).path("Temp").auto(),
        CleanableBuilder::env(ENV_WINDIR)
            .minimum_age(Duration::weeks(2))
            .path("Panther")
            .auto(),
        CleanableBuilder::env(ENV_WINDIR)
            .minimum_age(Duration::weeks(2))
            .path("Minidump")
            .auto(),
        CleanableBuilder::collection(PathCollections::User)
            .path(r"AppData\Local\Temp")
            .auto(),
        CleanableBuilder::collection(PathCollections::User)
            .path(r"AppData\Local\Microsoft\Windows\INetCache\IE")
            .auto(),
        CleanableBuilder::collection(PathCollections::User)
            .path(r"AppData\Local\Microsoft\Edge\User Data\Default\Cache")
            .auto(),
        CleanableBuilder::collection(PathCollections::User)
            .path(r"AppData\Local\Microsoft\Edge\User Data\Default\Code Cache")
            .auto(),
        CleanableBuilder::collection(PathCollections::User)
            .path(r"AppData\Local\Microsoft\Edge\User Data\Default\GPUCache")
            .auto(),
        CleanableBuilder::collection(PathCollections::User)
            .path(r"AppData\Local\Google\Chrome\User Data\Default\Cache")
            .auto(),
        CleanableBuilder::collection(PathCollections::User)
            .path(r"AppData\Local\Google\Chrome\User Data\Default\Code Cache")
            .auto(),
        CleanableBuilder::collection(PathCollections::User)
            .path(r"AppData\Local\Google\Chrome\User Data\Default\GPUCache")
            .auto(),
        CleanableBuilder::collection(PathCollections::User)
            .path(r"AppData\Local\Mozilla\Firefox\Profiles\*\*.default-release\cache2")
            .auto(),
        CleanableBuilder::collection(PathCollections::User)
            .path(r"AppData\Local\D3DSCache")
            .auto(),
        CleanableBuilder::collection(PathCollections::User)
            .path(r"AppData\Local\Nvidia\DXCache")
            .auto(),
        CleanableBuilder::collection(PathCollections::User)
            .path(r"AppData\Local\Microsoft\Windows\Exploerer\thumbcache_*")
            .auto(),
        CleanableBuilder::collection(PathCollections::User)
            .path(r"AppData\Local\Microsoft\Windows\Exploerer\iconcache_*")
            .auto(),
    ]
});

#[cfg(unix)]
pub static LOCATIONS: Lazy<Vec<CleanableBuilder>> = Lazy::new(Vec::new);

pub struct CleanablePath {
    pub paths: Vec<PathBuf>,
    pub auto: bool,
    pub minimum_age: Duration,
    pub duration_from: AgeType,
}

#[async_trait]
pub trait Indexed {
    async fn clean(&self, cli: &Flags) -> (usize, f64, usize, f64);
    async fn size(&self) -> u64;
}

#[async_trait]
impl Indexed for CleanablePath {
    async fn clean(&self, cli: &Flags) -> (usize, f64, usize, f64) {
        let mut auto_size = 0u64;
        let mut auto_files = 0usize;
        let mut manual_size = 0u64;
        let mut manual_files = 0usize;

        let mut collection: Vec<_> = self.paths.iter().flat_map(|p| p.collect()).collect();
        while let Some(buf) = collection.pop() {
            // TODO :: If all files in directory are removable call remove_dir, else call remove_file on each.
            if self.minimum_age.is_zero().not() {
                let metadata = match buf.metadata().context("Retrieve metadata for age check") {
                    Ok(m) => m,
                    Err(_) => {
                        error!("Failed to retrieve metadata for file: {}", buf.display());
                        continue;
                    }
                };

                let now = std::time::SystemTime::now();
                let from_time = match self.duration_from {
                    AgeType::FromAccess => metadata.accessed().unwrap(), // TODO :: Could error on some filesystems
                    AgeType::FromCreation => metadata.created().unwrap(),
                    AgeType::FromModification => metadata.modified().unwrap(),
                };

                let duration = now.duration_since(from_time).unwrap();
                if duration.as_millis() < self.minimum_age.num_milliseconds() as u128 {
                    trace!("File is not old enough to be removed: {}", buf.display());
                    continue;
                }

                trace!("File is old enough to be removed: {}", buf.display());
            }

            if self.auto.not() {
                trace!(
                    "File is not able to be automatically removed: {}",
                    buf.display()
                );

                manual_files += 1;
                manual_size += buf.metadata().unwrap().len();

                continue;
            }

            if cli.dry_run {
                info!("Would remove file: {:?}", buf);
                auto_size += buf.metadata().unwrap().len();
                auto_files += 1;
                continue;
            }

            match fs::remove_file(&buf) {
                Ok(_) => {
                    trace!("Removed file: {}", buf.display());
                    auto_size += buf.metadata().unwrap().len();
                    auto_files += 1;
                }
                Err(_) => error!("Failed to remove file: {}", buf.display()),
            }
        }

        // let size_after = self.size().await;
        // let size_cleaned = size - size_after;
        // let percent_cleaned = size_cleaned as f64 / size as f64 * 100.0;
        // percent_cleaned
        (
            auto_files,
            auto_size as f64,
            manual_files,
            manual_size as f64,
        )
    }

    async fn size(&self) -> u64 {
        let mut size = 0u64;
        for path in &self.paths {
            if let Ok(metadata) = fs::metadata(path) {
                size += metadata.len();
            }
        }

        size
    }
}

#[async_trait]
pub trait Collectable {
    fn collect(&self) -> Vec<PathBuf>;
}

#[async_trait]
impl Collectable for CleanablePath {
    fn collect(&self) -> Vec<PathBuf> {
        let mut inner_files = Vec::<PathBuf>::new();

        for path in &self.paths {
            inner_files.extend(path.collect());
        }

        inner_files
    }
}

#[async_trait]
impl Collectable for PathBuf {
    fn collect(&self) -> Vec<PathBuf> {
        let mut inner_files = Vec::<PathBuf>::new();
        let mut seen: HashSet<PathBuf> = HashSet::new();
        let mut issues = Vec::<(PathBuf, &str)>::new();
        let mut iter: Vec<PathBuf> = vec![];

        trace!("Collecting: {0}", self.display());

        let append = move |iter: &mut Vec<PathBuf>, parent: &PathBuf| {
            glob::glob(&format!("{0}/*", parent.display()))
                .unwrap()
                .for_each(|x| match x {
                    Ok(x) => iter.push(x),
                    Err(e) => error!("Failed to glob: {0}", e),
                });
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
