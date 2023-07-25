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

use anyhow::{Context, Result};
use cfg_if::cfg_if;
use std::fs;
use std::path::{Path, PathBuf};

cfg_if! {
    if #[cfg(windows)] {
        use std::sync::LazyLock;

        const PATH_SEPARATOR: char = '\\';
        const OTHER_PATH_SEPARATOR: char = '/';
        pub static SYSTEM_DRIVE: LazyLock<String> = LazyLock::new(|| {
            std::env::var("SystemDrive")
                .with_context(|| "Getting system drive from environment variable")
                .unwrap_or_else(|_| "C:".to_owned())
        });
    } else if #[cfg(unix)] {
        pub const PATH_SEPARATOR: char = '/';
        pub const OTHER_PATH_SEPARATOR: char = '\\';
        pub const SYSTEM_DRIVE: char = PATH_SEPARATOR;
    }
}

pub fn create_parents(path: &Path) -> Result<()> {
    path.parent().with_context(|| format!("Get parent directory for {}", &path.display())).and_then(|p| {
        fs::create_dir_all(p).with_context(|| format!("Creating parent directories for {}", &path.display()))
    })
}

pub fn normalise_path(path: PathBuf) -> PathBuf {
    let path = path.to_str().unwrap();

    // - all whitespace has been trimmed.
    // - all leading `/` has been trimmed.
    let path = path.trim().trim_start_matches(PATH_SEPARATOR);

    // Fast line for empty path.
    if path.is_empty() {
        #[cfg(windows)]
        return crate::windows::ROOT_DRIVE.clone();
        #[cfg(unix)]
        return PathBuf::from("/");
    }

    let path = path.replace(OTHER_PATH_SEPARATOR, PATH_SEPARATOR.to_string().as_str());

    let has_trailing = path.ends_with(PATH_SEPARATOR);

    let mut p = path.split(PATH_SEPARATOR).filter(|v| !v.is_empty());

    // Fuck you windows and your shitty filename limitations
    // TODO -> What else do we need to replace?
    let mut p = if cfg!(windows) {
        let drive = p.next().expect("Get drive root");
        let p = p.map(|v| v.replace(':', "_")).collect::<Vec<String>>().join(PATH_SEPARATOR.to_string().as_str());

        format!(
            "{drive}{sep}{path}",
            drive = drive.to_string(),
            sep = PATH_SEPARATOR,
            path = p
        )
    } else {
        let p = p.collect::<Vec<&str>>().join(PATH_SEPARATOR.to_string().as_str());
        format!("/{p}")
    };

    // Append trailing back if input path is ends-with `/`.
    if has_trailing {
        p.push(PATH_SEPARATOR);
    }

    PathBuf::from(p)
}
