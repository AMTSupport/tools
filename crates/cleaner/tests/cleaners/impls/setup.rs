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

use cleaner::cleaners::impls::USERS;
use cleaner::config::cli::Cli;
use cleaner::config::runtime::Runtime;
use std::fs;
use std::sync::{LazyLock, RwLock};

pub static RUNTIME: LazyLock<Runtime> = LazyLock::new(|| {
    use lib::cli::Flags;

    let cli = Cli {
        flags: Flags {
            verbose: 3,
            dry_run: true,
        },
        complete: None,
        cleaners: vec![],
    };

    let errors = RwLock::new(Vec::new());

    Runtime { cli, errors }
});

pub fn setup() {
    for user_paths in USERS.get_path() {
        fs::create_dir_all(user_paths).unwrap();
    }
}

pub fn teardown() {
    for user_paths in USERS.get_path() {
        fs::remove_dir_all(user_paths).unwrap();
    }
}

#[macro_export]
macro_rules! test {
    ($name:ident, $body:expr) => {
        #[test_log::test(tokio::test)]
        async fn $name() {
            use $crate::cleaners::impls::setup as _setup;

            let runtime = _setup::setup();
            $body
            _setup::teardown();
        }
    };
}

// pub fn with_runtime<F>(f: F)
// where
//     F: FnOnce(&Runtime),
// {
//     let (runtime) = setup();
//     f(&runtime);
//     teardown();
// }
