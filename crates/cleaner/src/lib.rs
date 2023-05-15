#![feature(thin_box)]

use anyhow::Context;
use std::boxed::ThinBox;
use std::fs::{metadata, File, Metadata};
use std::os::fd::AsFd;
use std::path::{Path, PathBuf};
use std::rt::panic_fmt;
use std::slice::Iter;
use std::time::Duration;

mod file;

#[cfg(windows)]
pub const ROOT_LOCATIONS: Vec<&str> = vec![];
#[cfg(unix)]
pub const ROOT_LOCATIONS: Vec<&str> = vec![];

#[cfg(windows)]
pub const USER_LOCATIONS: Vec<&str> = vec![];
#[cfg(unix)]
pub const USER_LOCATIONS: Vec<&str> = vec![];

// TODO :: Maybe change for windows since it reports sizes differently.
const KILOBYTE: u64 = 1024;
const MEGABYTE: u64 = 1024 * KILOBYTE;
const GIGABYTE: u64 = 1024 * MEGABYTE;

enum AgeType {
    FromCreation,
    FromModification,
    FromAccess,
}

pub struct SubPath {
    buf: PathBuf,
    auto: bool,
    minimum_age: Duration,
    duration_from: AgeType,
}

impl SubPath {
    fn new(buf: PathBuf, auto: bool, minimum_age: Duration, duration_from: AgeType) -> Self {
        Self {
            buf,
            auto,
            minimum_age,
            duration_from,
        }
    }

    fn new_auto(buf: PathBuf) -> Self {
        Self::new(buf, true, Duration::ZERO, AgeType::FromCreation)
    }

    fn new_manual(buf: PathBuf) -> Self {
        Self::new(buf, false, Duration::ZERO, AgeType::FromCreation)
    }

    fn new_aged(buf: PathBuf, minimum_age: Duration, duration_from: AgeType) -> Self {
        assert!(
            minimum_age > Duration::ZERO,
            "minimum_age must be greater than zero"
        );

        // TODO :: Assert that FS/OS supports the duration_from type

        Self::new(buf, true, minimum_age, duration_from)
    }
}

trait Collectable {
    fn collect(&self) -> (Vec<PathBuf>, Vec<PathBuf>, Vec<PathBuf>);
}

impl Collectable for Path {
    fn collect(&self) -> (Vec<PathBuf>, Vec<PathBuf>, Vec<PathBuf>) {
        let mut inner_files = Vec::<PathBuf>::new();
        let mut inner_directories = Vec::<PathBuf>::new();
        let mut issues = Vec::<PathBuf>::new();

        for entry in self.read_dir().context("Read directory")? {
            let entry = entry.unwrap();
            let path = entry.path();
            let meta = path.metadata().unwrap();

            if meta.is_symlink() {
                trace!("Skipping symlink: {:?}", inner);
                continue;
            }

            if meta.is_dir() {
                trace!("Found directory: {:?}", inner);
                inner_directories.push(*meta);
                continue;
            }

            if meta.is_file() {
                trace!("Found file: {:?}", inner);
                inner_files.push(*meta);
                continue;
            }

            trace!("Issue with: {:?}", inner);
            issues.push(entry.path());
        }

        return (inner_files, inner_directories, issues);
    }
}

trait Composable {
    fn compose_all(&self, mut root_bufs: Iter<PathBuf>) -> Vec<ThinBox<dyn Composed>> {
        root_bufs
            .filter_map(|root_buf| self.compose(root_buf).ok())
            .collect()
    }

    fn compose(&self, root_buf: &PathBuf) -> Some(ThinBox<dyn Composed>);
}

impl Composable for SubPath {
    fn compose(&self, root_buf: &PathBuf) -> Some(ThinBox<dyn Composed>) {
        if self.is_absolute() {
            error!("PathBuf is not relative: {:?}", self);
            return None;
        }

        let path = root_buf.join(self);
        if !path.exists() {
            error!("Composed path `{0}` doesn't exist.", path);
            return None;
        }

        if !path.is_dir() {
            error!("Composed path `{0}` is not a directory.", path);
            return None;
        }

        return Some(ThinBox::new(Composed::new(&self, path.into_boxed_path())));
    }
}

trait Composed {
    fn new(path_base: &SubPath, path: Box<Path>) -> Self;
    fn clean(&self) -> bool;
    fn size(&self) -> u64;
}

struct ComposedStruct<'a> {
    path_base: &'a SubPath,
    path: Box<Path>,

    pub files: Vec<PathBuf>,
    pub directories: Vec<PathBuf>,
}

impl Composed for ComposedStruct {
    fn new(path_base: &SubPath, path: Box<Path>) -> Self {
        let (files, directories, issues) = path.collect();

        if !issues.is_empty() {
            let mapped_issues = issues
                .iter()
                .map(|issue| {
                    format!(
                        "\t{}",
                        issue
                            .into_path_buf()
                            .to_str()?
                            .strip_prefix(path.to_str().unwrap())
                            .unwrap()
                    )
                })
                .collect::<Vec<String>>()
                .join("\n");

            error!(
                "{0} contains {1} issues.\n{2}",
                path,
                issues.len(),
                mapped_issues
            );
        }

        Self {
            path_base,
            path,
            files,
            directories,
        }
    }

    fn clean(&self) -> bool {
        let size = self.size();
        let size_str = if size >= GIGABYTE {
            format!("{:.2} GB", size as f64 / GIGABYTE)
        } else if size >= MEGABYTE {
            format!("{:.2} MB", size as f64 / MEGABYTE)
        } else if size >= KILOBYTE {
            format!("{:.2} KB", size as f64 / KILOBYTE)
        } else {
            format!("{:.2} B", size)
        };

        info!(
            "Cleaning {} files from {}, cleaning up {}",
            self.files.len(),
            self.path,
            size_str
        );

        let mut paired: Vec<(PathBuf, Metadata)> = self
            .files
            .iter()
            .map(|buf| (buf, buf.metadata().unwrap()))
            .collect();
        while let Some((buf, meta)) = paired.pop() {
            // TODO :: Permission checks
            // TODO :: Lock checks
            // TODO :: File open checks

            if
        }

        while let Some(meta) = metas.pop() {
            if meta.is_symlink() {
                trace!("Skipping symlink: {:?}", inner);
                continue;
            }

            if meta.is_dir() {
                directories.push(meta);
                continue;
            }

            if meta.is_file() {
                if let Err(e) = std::fs::remove_file(meta.path()) {
                    error!("Failed to remove file: {:?}", e);
                    continue;
                }
            }
        }

        while let Some(meta) = directories.pop() {
            if let Err(e) = std::fs::remove_dir(meta.path()) {
                error!("Failed to remove directory: {:?}", e);
                continue;
            }
        }

        return true;
    }

    fn size(&self) -> u64 {
        todo!()
    }
}

// impl Cleanable for ManualComposed {
//     fn clean(&self) -> bool {
//         let mut metas = self.collect();
//         let mut directories = Vec::new();
//
//         while let Some(meta) = metas.pop() {
//             if meta.is_symlink() {
//                 trace!("Skipping symlink: {:?}", inner);
//                 continue;
//             }
//
//             if meta.is_dir() {
//                 directories.push(meta);
//                 continue;
//             }
//
//             if meta.is_file() {
//                 if let Err(e) = std::fs::remove_file(meta.path()) {
//                     error!("Failed to remove file: {:?}", e);
//                     continue;
//                 }
//             }
//         }
//
//         while let Some(meta) = directories.pop() {
//             if let Err(e) = std::fs::remove_dir(meta.path()) {
//                 error!("Failed to remove directory: {:?}", e);
//                 continue;
//             }
//         }
//
//         return true;
//     }
// }

// trait AutoComposed {}

/// A type of Composed that will be scanned but will require user input to be cleaned.
// trait ManualComposed where Self: AutoComposed {}

impl Composable for PathBuf {
    fn compose(&self, root_buf: &PathBuf) -> Some<ThinBox<dyn Composed>> {
        let connected = root_buf.join(self);
        if connected.exists() {
            return Ok(ThinBox::<dyn Composed>::new(Composed::new(connected)));
        }

        trace!("Skipping non-existent composable path `{:?}`", connected);
        return None;
    }
}

impl RecursiveSize for PathBuf {
    fn size(&self) -> u128 {
        let mut total_size: u128 = 0;

        #[cfg(target_family = "unix")]
        use std::os::unix::fs::MetadataExt;
        #[cfg(target_os = "windows")]
        use std::os::windows::fs::MetadataExt;

        for file in files {
            total_size += file.size();
        }

        return total_size;
    }
}

impl Cleanable for ComposedPath {
    fn clean(&self) -> bool {
        let mut metas = self.collect();
        let mut directories = Vec::new();

        while let Some(meta) = metas.pop() {
            if meta.is_symlink() {
                trace!("Skipping symlink: {:?}", inner);
                continue;
            }

            if meta.is_dir() {
                trace!("Found directory: {:?}", inner);
                directories.extend(inner);
                continue;
            }

            if meta.is_file() {
                trace!("Found file: {:?}", inner);
            }
        }

        return false;
    }
}

pub struct AutoCleanable
where
    Self: Path,
    Self: Cleanable,
    Self: RecursiveSize,
{
    path: dyn Path,
}

pub struct ManualCleanable
where
    Self: Path,
    Self: Cleanable,
    Self: RecursiveSize,
{
    path: dyn Path,
}
