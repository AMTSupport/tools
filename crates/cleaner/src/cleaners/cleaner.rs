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

use crate::cleaners::location::Location;
use crate::rule::Rules;
use macros::{Delegation, EnumVariants};
use std::fmt::Debug;
use std::path::Path;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, EnumVariants, Delegation)]
#[delegate(trait = CleanerInternal)]
pub enum Cleaner {
    #[delegate(path = crate::cleaners::impls::log::LogCleaner)]
    Logs,
    #[delegate(path = crate::cleaners::impls::browser::BrowserCleaner)]
    Browsers,
    // Environment,
    // Cache,
    // Temporary,
    // Downloads,
    // Trash,
    // RecycleBin,
    // Thumbnails,
    // CrashDumps,
    // OldWindows,
}

// impl Cleaner {
//     const LOGS: LazyCell<Box<dyn CleanerInternal>> = LazyCell::new(|| {
//         Box::new(crate::cleaners::impls::log::LogCleaner::new())
//     });
//     const BROWSERS: LazyCell<Box<dyn CleanerInternal>> = LazyCell::new(|| {
//         Box::new(crate::cleaners::impls::browser::BrowserCleaner::new())
//     });
//
//     pub fn delegate(&self) -> LazyCell<Box<dyn CleanerInternal>, fn() -> Box<dyn CleanerInternal>> {
//         match self {
//             Cleaner::Logs => Cleaner::LOGS,
//             Cleaner::Browsers => Cleaner::BROWSERS,
//         }
//     }
// }

pub trait CleanerInternal: Send + Sync + Debug + 'static {
    fn new() -> Self
    where
        Self: Sized;

    fn rules(&self) -> Rules;

    fn locations(&self) -> Vec<Location>;

    /// Returns if the cleaner is able to be run on the current platform.
    fn supported(&self) -> bool {
        true
    }

    fn clean(&self) -> CleanupResult {
        CleanupResult::Skipped(format!("{} is not implemented", std::any::type_name::<Self>()))
    }
}

#[derive(Debug)]
pub enum CleanupResult {
    Cleaned(u64),
    Partial {
        cleaned: u64,
        missed_files: Vec<&'static Path>,
    },
    Skipped(String),
    Failed(anyhow::Error),
}
