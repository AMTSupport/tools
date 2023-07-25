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
#![feature(lazy_cell)]
#![feature(result_option_inspect)]
#![feature(return_position_impl_trait_in_trait)]
#![feature(type_alias_impl_trait)]
#![feature(trait_alias)]
#![feature(downcast_unchecked)]
#![feature(const_trait_impl)]
#![feature(impl_trait_in_assoc_type)]
#![feature(thin_box)]
#![feature(associated_type_defaults)]

pub mod application;
pub mod builder;
pub mod cleaners;
pub mod config;
pub mod rule;

// #[derive(Debug, Clone)]
// pub enum PathCollections {
//     Drive,
//     User,
// }
//
// #[cfg(windows)]
// const ENV_PROGRAMDATA: &str = "ProgramData";
// #[cfg(windows)]
// const ENV_WINDIR: &str = "windir";
//
// #[cfg(windows)]
// pub static LOCATIONS: Lazy<Vec<CleanableBuilder>> = Lazy::new(|| {
//     vec![
//         CleanableBuilder::collection(PathCollections::Drive)
//             .paths("$Recycle.Bin")
//             .auto()
//             .minimum_age(Duration::weeks(1))
//             .duration_from(AgeType::FromModification),
//         CleanableBuilder::env(ENV_PROGRAMDATA).paths("NVIDIA").auto(),
//         CleanableBuilder::env(ENV_PROGRAMDATA).paths("Nvidia Corporation/Downloader").auto(),
//         CleanableBuilder::env(ENV_PROGRAMDATA).paths("Microsoft/Windows/WER/ReportArchive").auto(),
//         CleanableBuilder::env(ENV_PROGRAMDATA).paths("NinitePro/NiniteDownloads/Files").auto(),
//         CleanableBuilder::env(ENV_WINDIR).paths("Downloaded Program Files").auto(),
//         CleanableBuilder::env(ENV_WINDIR).paths("SoftwareDistribution/Download"),
//         CleanableBuilder::env(ENV_WINDIR).paths("Prefetch").auto(),
//         CleanableBuilder::env(ENV_WINDIR).paths("Temp").auto(),
//         CleanableBuilder::env(ENV_WINDIR)
//             .minimum_age(Duration::weeks(2))
//             .paths("Panther")
//             .duration_from(AgeType::FromModification)
//             .auto(),
//         CleanableBuilder::env(ENV_WINDIR).minimum_age(Duration::weeks(1)).paths("Minidump").auto(),
//         CleanableBuilder::collection(PathCollections::User).paths("AppData/Local/Temp").auto(),
//         CleanableBuilder::collection(PathCollections::User)
//             .paths("AppData/Local/Microsoft/Windows/INetCache/IE")
//             .auto(),
//         CleanableBuilder::collection(PathCollections::User)
//             .paths("AppData/Local/Microsoft/Edge/User Data/Default/Cache")
//             .auto(),
//         CleanableBuilder::collection(PathCollections::User)
//             .paths("AppData/Local/Microsoft/Edge/User Data/Default/Code Cache")
//             .auto(),
//         CleanableBuilder::collection(PathCollections::User)
//             .paths("AppData/Local/Microsoft/Edge/User Data/Default/GPUCache")
//             .auto(),
//         CleanableBuilder::collection(PathCollections::User)
//             .paths("AppData/Local/Google/Chrome/User Data/Default/Cache")
//             .auto(),
//         CleanableBuilder::collection(PathCollections::User)
//             .paths("AppData/Local/Google/Chrome/User Data/Default/Code Cache")
//             .auto(),
//         CleanableBuilder::collection(PathCollections::User)
//             .paths("AppData/Local/Google/Chrome/User Data/Default/GPUCache")
//             .auto(),
//         CleanableBuilder::collection(PathCollections::User)
//             .paths("AppData/Local/Mozilla/Firefox/Profiles/*.default-release/cache2")
//             .auto(),
//         CleanableBuilder::collection(PathCollections::User).paths("AppData/Local/D3DSCache").auto(),
//         CleanableBuilder::collection(PathCollections::User)
//             .paths("AppData/Local/NVIDIA/DXCache")
//             .auto(),
//         // TODO :: Figure out how to clean these without restart explorer.exe
//         CleanableBuilder::collection(PathCollections::User)
//             .paths("AppData/Local/Microsoft/Windows/Explorer/thumbcache_*")
//             .auto(),
//         CleanableBuilder::collection(PathCollections::User)
//             .paths("AppData/Local/Microsoft/Windows/Explorer/iconcache_*")
//             .auto(),
//     ]
// });
//
// #[cfg(unix)]
// pub static LOCATIONS: LazyCell<Vec<CleanableBuilder>> = LazyCell::new(Vec::new);
//
// pub struct CleanablePath {
//     pub base_buf: String,
//     pub paths: Vec<PathBuf>,
//     pub auto: bool,
//     pub minimum_age: Duration,
//     pub duration_from: AgeType,
// }
//
// pub trait Indexed {
//     fn prepare(
//         &self,
//         cli: &Flags,
//         bar: ProgressBar,
//     ) -> anyhow::Result<(PreparedPaths, PreparedPaths)>;
// }
//
// #[derive(Default, Clone, Debug)]
// pub struct PreparedPaths {
//     paths: Vec<PathBuf>,
//     disk_size: u64,
// }
//
// impl PreparedPaths {
//     fn merge_with(&mut self, other: PreparedPaths) {
//         self.paths.extend(other.paths);
//         self.disk_size += other.disk_size;
//     }
// }
//
// impl Indexed for CleanablePath {
//     fn prepare(
//         &self,
//         _cli: &Flags,
//         bar: ProgressBar,
//     ) -> anyhow::Result<(PreparedPaths, PreparedPaths)> {
//         let mut prepared_auto = PreparedPaths {
//             paths: Vec::new(),
//             disk_size: 0,
//         };
//         let mut prepared_manual = PreparedPaths {
//             paths: Vec::new(),
//             disk_size: 0,
//         };
//
//         let bar = bar.with_message(format!("Collecting files from {}", self.base_buf));
//         let iter = self.paths.iter().flat_map(|paths| paths.collect()).progress_with(bar);
//
//         for buf in iter {
//             if self.newer_than_allowed(&buf)? {
//                 trace!("File is newer than allowed: {}", buf.display());
//                 continue;
//             }
//
//             if self.auto.not() {
//                 trace!("File is not auto cleanable: {}", buf.display());
//                 prepared_manual.disk_size += buf.metadata()?.len();
//                 prepared_manual.paths.push(buf);
//                 continue;
//             }
//
//             prepared_auto.disk_size += buf.metadata()?.len();
//             prepared_auto.paths.push(buf);
//         }
//
//         Ok((prepared_auto, prepared_manual))
//     }
// }
//
// impl CleanablePath {
//     fn newer_than_allowed(&self, buf: &Path) -> anyhow::Result<bool> {
//         // There is no minimum age set, so it's allowed to be removed.
//         if self.minimum_age.is_zero() {
//             return Ok(false);
//         }
//
//         let metadata = buf
//             .metadata()
//             .with_context(|| format!("Retrieve metadata for age check: {}", buf.display()))?;
//         let now = std::time::SystemTime::now();
//         let from_time = match self.duration_from {
//             AgeType::FromAccess => metadata
//                 .accessed()
//                 .with_context(|| format!("Get last accessed age for {}", buf.display()))?, // TODO :: Could error on some filesystems
//             AgeType::FromCreation => metadata
//                 .created()
//                 .with_context(|| format!("Get creation age for {}", buf.display()))?,
//             AgeType::FromModification => metadata
//                 .modified()
//                 .with_context(|| format!("Get last modified age for {}", buf.display()))?,
//         };
//
//         let duration = now
//             .duration_since(from_time)
//             .with_context(|| format!("Calculate duration for {}", buf.display(),))?;
//
//         Ok(duration.as_millis() < self.minimum_age.num_milliseconds() as u128)
//     }
// }
//
// pub trait Collectable {
//     fn collect(&self) -> Vec<PathBuf>;
// }
//
// impl Collectable for CleanablePath {
//     fn collect(&self) -> Vec<PathBuf> {
//         let mut inner_files = Vec::<PathBuf>::new();
//
//         for paths in &self.paths {
//             inner_files.extend(paths.collect());
//         }
//
//         inner_files
//     }
// }
//
// impl Collectable for PathBuf {
//     fn collect(&self) -> Vec<PathBuf> {
//         let mut inner_files = Vec::<PathBuf>::new();
//         let mut seen: HashSet<PathBuf> = HashSet::new();
//         let mut issues = Vec::<(PathBuf, &str)>::new();
//         let mut iter: Vec<PathBuf> = vec![];
//
//         trace!("Collecting: {0}", self.display());
//
//         let append = move |iter: &mut Vec<PathBuf>, parent: &PathBuf| match globbed(parent) {
//             None => {
//                 trace!("No globbed files found: {0}", parent.display());
//                 iter.push(parent.clone());
//             }
//             Some(glob) => {
//                 trace!(
//                     "Found {num} globbed files for {parent}",
//                     num = glob.len(),
//                     parent = parent.display()
//                 );
//
//                 iter.extend(glob);
//             }
//         };
//
//         append(&mut iter, self);
//
//         while let Some(paths) = iter.pop() {
//             if seen.get(&paths).is_none() {
//                 seen.insert(paths.clone());
//             } else {
//                 continue;
//             }
//
//             let meta = paths.metadata().unwrap();
//
//             if meta.is_symlink() {
//                 trace!("Skipping symlink: {0}", &paths.display());
//                 issues.push((paths.clone(), "Symlink"));
//                 continue;
//             }
//
//             #[cfg(unix)]
//             if permissions::is_removable(&paths).is_ok_and(|x| !x) {
//                 trace!("Skipping non-removable: {0}", &paths.display());
//                 issues.push((paths.clone(), "Non-removable"));
//             }
//
//             #[cfg(windows)] // TODO :: Implement this for windows
//             if false {
//                 trace!("Skipping non-removable: {0}", &paths.display());
//                 issues.push((paths.clone(), "Non-removable"));
//             }
//
//             if meta.is_dir() {
//                 trace!("Found directory: {0}", &paths.display());
//                 append(&mut iter, &paths);
//                 continue;
//             }
//
//             if meta.is_file() {
//                 trace!("Found file: {0}", &paths.display());
//                 inner_files.push(paths.clone());
//                 continue;
//             }
//
//             trace!("Issue with: {0}", &paths.display());
//             issues.push((paths.clone(), "Unknown issue"));
//         }
//
//         if !issues.is_empty() {
//             let mapped_issues = issues
//                 .iter()
//                 .map(|issue| {
//                     format!(
//                         "\t{1}: {0}",
//                         issue.1,
//                         issue.0.to_path_buf().to_str().unwrap()
//                     )
//                 })
//                 .collect::<Vec<String>>()
//                 .join("\n");
//
//             warn!("Issues with {0}:\n{1}", self.display(), mapped_issues);
//         }
//
//         inner_files
//     }
// }
//
// pub(crate) fn globbed(buf: &PathBuf) -> Option<Vec<PathBuf>> {
//     let glob_str = match buf {
//         buf if buf.exists() && buf.is_dir() => format!("{0}/*", buf.to_str().unwrap()),
//         buf if buf.exists() && buf.is_file() => return Some(vec![buf.clone()]),
//         buf => format!("{buf}/*", buf = buf.to_str().unwrap()),
//     };
//
//     debug!("Globbing: {0}", &glob_str);
//
//     match glob::glob(&glob_str) {
//         Err(e) => {
//             error!("Failed to glob: {0}", e);
//             None
//         }
//         Ok(paths) => Some(paths.filter_map(|x| x.ok()).collect::<Vec<PathBuf>>()),
//     }
// }
//
// pub(crate) fn clean(prepared: &PreparedPaths) -> anyhow::Result<u64> {
//     // TODO :: If all files in directory are removable call remove_dir, else call remove_file on each.
//     Ok(prepared
//         .paths
//         .par_iter()
//         .progress_count(prepared.paths.len() as u64)
//         .map(|buf| {
//             match std::fs::remove_file(buf).with_context(|| format!("Delete {}", buf.display())) {
//                 Ok(_) => {
//                     trace!("Deleted: {0}", buf.display());
//                     0
//                 }
//                 Err(e) => {
//                     trace!("Failed to delete: {0} ({1})", buf.display(), e);
//                     buf.metadata().unwrap().len()
//                 }
//             }
//         })
//         .sum())
// }
