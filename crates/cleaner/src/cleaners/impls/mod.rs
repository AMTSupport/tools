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
use cfg_if::cfg_if;
use std::sync::LazyLock;

pub mod browser;
pub mod downloads;
pub mod log;
pub mod shader;
pub mod temp;
pub mod thumbnail;
pub mod trash;

cfg_if! {
    if #[cfg(unix)] {
        pub static USERS: LazyLock<Location> = LazyLock::new(|| Location::Globbing("/home/*/".into()));
    } else if #[cfg(windows)] {
        use crate::cleaners::env_dir;
        use std::path::PathBuf;

        pub static PROGRAM_DATA: LazyLock<PathBuf> = LazyLock::new(|| env_dir("ProgramData".into()).expect("This is always set on Windows"));
        pub static WINDIR: LazyLock<PathBuf> = LazyLock::new(|| env_dir("windir".into()).expect("This is always set on Windows"));
        pub static USERS: LazyLock<Location> = LazyLock::new(|| {
            let mut path = env_dir("SystemDrive".to_owned()).unwrap();
            path.push("\\Users");
            path.push("*");
            Location::Globbing(path.to_string_lossy().to_string())
        });
    } else {
        compile_error!("Unsupported platform");
    }
}
