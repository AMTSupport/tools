/*
 * Copyright (C) 2026 James Draycott <me@racci.dev>
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

use std::env;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, SystemTime};
use tracing::warn;

/// Set once from the CLI to ignore existing cache entries for the rest of the process.
static BYPASS: AtomicBool = AtomicBool::new(false);

/// Ignores the cache for the rest of this process; fresh downloads still refresh it.
pub fn set_bypass(enabled: bool) {
    BYPASS.store(enabled, Ordering::Relaxed);
}

pub fn bypassed() -> bool {
    BYPASS.load(Ordering::Relaxed)
}

/// Returns this application's cache directory, creating it if missing.
pub fn dir() -> Option<PathBuf> {
    let root = env::var_os("XDG_CACHE_HOME")
        .map(PathBuf::from)
        .or_else(|| env::var_os("LOCALAPPDATA").map(PathBuf::from))
        .or_else(|| env::var_os("HOME").map(|home| PathBuf::from(home).join(".cache")))?;

    let dir = root.join("country-ip");
    fs_err::create_dir_all(&dir).ok()?;

    Some(dir)
}

/// Reads a cached file, treating unreadable and stale files as absent.
pub fn read(path: &Path, max_age: Duration) -> Option<String> {
    if bypassed() {
        return None;
    }

    let modified = fs_err::metadata(path).ok()?.modified().ok()?;
    if !is_fresh(modified, max_age) {
        return None;
    }

    fs_err::read_to_string(path).ok()
}

/// Reads cache off-thread so async callers do not block runtime worker threads.
pub async fn read_async(path: PathBuf, max_age: Duration) -> Option<String> {
    tokio::task::spawn_blocking(move || read(&path, max_age)).await.ok().flatten()
}

/// Writes a file atomically so an interrupted run cannot leave a truncated cache behind.
pub fn write(path: &Path, contents: &str) {
    if let Some(parent) = path.parent() {
        if let Err(err) = fs_err::create_dir_all(parent) {
            warn!("Failed to create cache directory {}: {err}", parent.display());
            return;
        }
    }

    let temp = path.with_extension(format!("tmp-{}", std::process::id()));
    if let Err(err) = fs_err::write(&temp, contents) {
        warn!("Failed to write cache file {}: {err}", temp.display());
        return;
    }

    if let Err(err) = fs_err::rename(&temp, path) {
        warn!("Failed to move cache file into place at {}: {err}", path.display());
        let _ = fs_err::remove_file(&temp);
    }
}

/// A file is fresh when its modification time is no older than `max_age`.
/// Timestamps in the future are treated as stale so a bad clock cannot pin a dump forever.
pub fn is_fresh(modified: SystemTime, max_age: Duration) -> bool {
    SystemTime::now().duration_since(modified).is_ok_and(|age| age < max_age)
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;

    const DAY: Duration = Duration::from_secs(24 * 60 * 60);

    fn scratch(name: &str) -> PathBuf {
        env::temp_dir()
            .join(format!("country-ip-cache-test-{}", std::process::id()))
            .join(name)
    }

    #[test]
    fn fresh_recent_timestamps() {
        assert!(is_fresh(SystemTime::now(), DAY));
    }

    #[test]
    fn stale_old_timestamps() {
        let two_days_ago = SystemTime::now() - Duration::from_secs(2 * 24 * 60 * 60);
        assert!(!is_fresh(two_days_ago, DAY));
    }

    #[test]
    fn future_timestamps_are_stale() {
        let tomorrow = SystemTime::now() + DAY;
        assert!(!is_fresh(tomorrow, DAY));
    }

    #[test]
    fn round_trips_contents() -> Result<()> {
        let path = scratch("round-trip/file.txt");

        write(&path, "hello");
        assert_eq!(read(&path, DAY), Some(String::from("hello")));
        assert_eq!(read(&path, Duration::ZERO), None);
        assert_eq!(read(&scratch("round-trip/missing.txt"), DAY), None);

        fs_err::remove_dir_all(scratch("round-trip"))?;
        Ok(())
    }
}
