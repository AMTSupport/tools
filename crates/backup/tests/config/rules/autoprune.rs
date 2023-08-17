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

use anyhow::Result;
use rand::random;
use std::fs::File;
use std::path::PathBuf;
use tempdir::TempDir;
use backup::config::rules::autoprune::AutoPrune;

fn setup() -> Result<TempDir> {
    let dir = TempDir::new("test")?;
    Ok(dir)
}

fn get_file(dir: &TempDir) -> Result<(File, PathBuf)> {
    let file_path = dir.path().join(random());
    let tmp_file = File::create(&file_path)?;

    Ok((tmp_file, file_path))
}

#[test_log::test(test)]
fn autoprune_disabled_works() -> Result<()> {
    // Would prune everything if enabled.
    let rules = AutoPrune {
        enabled: false,
        keep_latest: 0,
        days: 0,
        weeks: 0,
        months: 0,
    };

    // let rules = Rules::default();
    // assert_eq!(rules.auto_prune.enabled, false);

    let tmp_dir = setup()?;
    let tmp_file = get_file(&tmp_dir)?;
    assert_eq!(rules.auto_prune.should_prune(&tmp_file, 1), Ok(false));

    Ok(())
}

#[test_log::test(test)]
fn autoprune_enabled_works() -> Result<()> {
    let prune = AutoPrune {
        enabled: true,
        keep_latest: 0,
        days: 0,
        weeks: 0,
        months: 0,
    };

    let tmp_dir = setup()?;
    let (_, path) = get_file(&tmp_dir)?;
    assert_eq!(prune.should_prune(&path, 1), Ok(true));

    Ok(())
}

#[test_log::test(test)]
fn autoprune_enabled_keep_latest_works() -> Result<()> {
    let rule = AutoPrune {
        enabled: true,
        keep_latest: 1,
        days: 0,
        weeks: 0,
        months: 0,
    };

    let tmp_dir = setup()?;
    let (tmp_file, _) = get_file(&tmp_dir)?;
    assert_eq!(rule.should_prune(&tmp_file, 1), Ok(false));

    Ok(())
}
