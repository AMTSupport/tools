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
use crate::cleaners::impls::USERS;
use crate::cleaners::location::Location;
use crate::config::runtime::Runtime;
use crate::rule::Rules;
use async_trait::async_trait;

#[derive(Default, Debug, Clone, Copy)]
pub struct BrowserCleaner;

// TODO: Support all profiles for browsers. (I only use firefox)
#[async_trait]
impl CleanerInternal for BrowserCleaner {
    fn rules(&self) -> Rules {
        vec![]
    }

    #[cfg(unix)]
    fn locations(&self) -> Vec<Location> {
        vec![
            Location::Sub(&USERS, ".cache/mozilla/firefox/*/cache2/*".to_string()),
            Location::Sub(&USERS, ".cache/google-chrome/Default/Cache/*".to_string()),
        ]
    }

    #[cfg(windows)]
    fn locations(&self) -> Vec<Location> {
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

    async fn clean(&self, runtime: &'static Runtime) -> CleanupResult {
        basic_files(Cleaner::Browsers, self, runtime).await
    }
}
