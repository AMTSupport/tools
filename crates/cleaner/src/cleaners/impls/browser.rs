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

use crate::cleaners::cleaner::{Cleaner, CleanerInternal, CleanupResult};
use crate::cleaners::location::Location;
use crate::config::runtime::Runtime;
use crate::rule::Rules;

#[derive(Default, Debug, Clone, Copy)]
pub struct BrowserCleaner;

impl CleanerInternal for BrowserCleaner {
    fn new() -> Self
    where
        Self: Sized,
    {
        Self::default()
    }

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
        let prefix = "AppData/Local/";

        vec![
            Location::Sub(&USERS, format!("{prefix}Microsoft/Windows/INetCache/IE/*")),
            Location::Sub(&USERS, format!("{prefix}Microsoft/Edge/User Data/Default/Cache/*")),
            Location::Sub(&USERS, format!("{prefix}Google/Chrome/User Data/Default/Cache/*")),
            Location::Sub(
                &USERS,
                format!("{prefix}Mozilla/Firefox/Profiles/*.default-release/cache2/*"),
            ),
        ]
    }

    fn clean(&self, runtime: &'static Runtime) -> CleanupResult {
        use crate::cleaners::cleaner::{clean_files, collect_locations};

        let (passed, failed) = collect_locations(self.locations(), self.rules());
        let passed_result = clean_files(Cleaner::Browsers, passed, &runtime);
        let final_result = passed_result.extend_missed(failed);

        final_result
    }
}
