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

use crate::cleaners::cleaner::Cleaner::Trash;
use crate::cleaners::cleaner::{CleanerInternal, CleanupResult};
use crate::cleaners::location::Location;
use crate::config::runtime::Runtime;
use crate::rule::{age::Since, Rule, Rules};
use chrono::Duration;

#[derive(Default, Debug, Clone, Copy)]
pub struct TrashCleaner;

impl CleanerInternal for TrashCleaner {
    fn new() -> Self
    where
        Self: Sized,
    {
        Self::default()
    }

    /// Only clean items that were added more than 7 days ago.
    fn rules(&self) -> Rules {
        vec![Rule::Age(Duration::days(7), Since::Modified)]
    }

    #[cfg(unix)]
    fn locations(&self) -> Vec<Location> {
        vec![]
    }

    #[cfg(windows)]
    fn locations(&self) -> Vec<Location> {
        use super::{PROGRAM_DATA, WINDIR};
        use lib::fs::DRIVES;

        DRIVES
            .iter()
            .map(|drive| Location::Globbing(drive.join("$RECYCLE.BIN/*").to_string_lossy().to_string()))
            .collect()
    }

    fn clean(&self, runtime: &'static Runtime) -> CleanupResult {
        use crate::cleaners::cleaner::{clean_files, collect_locations};

        let (passed, failed) = collect_locations(self.locations(), self.rules());
        let passed_result = clean_files(Trash, passed, &runtime);
        let final_result = passed_result.extend_missed(failed);

        final_result
    }
}
