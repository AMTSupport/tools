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

use crate::cleaners::cleaner::{basic_files, Cleaner, CleanerInternal, CleanupResult};
use crate::cleaners::location::Location;
use crate::config::runtime::Runtime;
use crate::rule::Rules;
use async_trait::async_trait;

#[derive(Default, Debug, Clone, Copy)]
pub struct ThumbnailCleaner;

#[async_trait]
impl CleanerInternal for ThumbnailCleaner {
    fn rules(&self) -> Rules {
        vec![]
    }

    #[cfg(unix)]
    fn locations(&self) -> Vec<Location> {
        vec![]
    }

    #[cfg(windows)]
    fn locations(&self) -> Vec<Location> {
        use super::USERS;
        let prefix = "AppData/Local/Microsoft/Windows/Explorer/";

        vec![
            Location::Sub(&USERS, format!("{prefix}thumbcache_*")),
            Location::Sub(&USERS, format!("{prefix}iconcache_*")),
        ]
    }

    async fn clean(&self, runtime: &'static Runtime) -> CleanupResult {
        basic_files(Cleaner::Thumbnails, self, runtime).await
    }
}
