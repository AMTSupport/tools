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
pub struct DownloadsCleaner;

// TODO - Browser Shader Cache
#[async_trait]
impl CleanerInternal for DownloadsCleaner {
    fn rules(&self) -> Rules {
        vec![]
    }

    #[cfg(unix)]
    fn locations(&self) -> Vec<Location> {
        vec![]
    }

    #[cfg(windows)]
    fn locations(&self) -> Vec<Location> {
        use super::{PROGRAM_DATA, WINDIR};
        use lib::fs::SYSTEM_DRIVE;

        vec![
            // Windows Update / Prefetch files
            Location::Globbing(WINDIR.join("Downloaded Program Files/*").to_string_lossy().into()),
            Location::Globbing(WINDIR.join("SoftwareDistribution/Download/*").to_string_lossy().into()),
            Location::Globbing(WINDIR.join("Prefetch/*").to_string_lossy().into()),
            // Graphic Drivers
            Location::Globbing(PROGRAM_DATA.join("NVIDIA Corporation/Downloader/*").to_string_lossy().into()),
            // Package Managers
            Location::Globbing(SYSTEM_DRIVE.join("NinitePro/NiniteDownloads/Files/*").to_string_lossy().into()),
        ]
    }

    async fn clean(&self, runtime: &'static Runtime) -> CleanupResult {
        basic_files(Cleaner::Downloads, self, runtime).await
    }
}
