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

use crate::rule::{Rule, RuleError};
use chrono::Duration;
use std::path::Path;
use tracing::{debug, error, trace};

#[derive(Debug, Clone, Copy)]
pub enum Since {
    Accessed,
    Modified,
    Creation,
}

pub(super) fn test(rule: Rule, path: &Path) -> bool {
    let (duration, since) = match rule {
        Rule::Age(duration, since) => (duration, since),
        _ => panic!("This shouldn't happen"),
    };

    let meta = match super::meta(path) {
        Ok(m) => m,
        Err(err) => {
            error!("Failed to get file {} metadata: {err}", path.display());
            return false;
        }
    };

    use Since::*;
    let accessed = match since {
        Accessed { .. } => meta.accessed(),
        Modified { .. } => meta.modified(),
        Creation { .. } => meta.created(),
    }
    .inspect(|date| trace!("File {} was {date:?}", path.display()))
    .inspect_err(|err| error!("Failed to get file {} date: {err}", path.display()))
    .map(|d| d.elapsed().unwrap())
    .map(|d| Duration::from_std(d).unwrap())
    .map_err(|err| RuleError::MetadataError(err, path.to_path_buf()));

    let from_date = match accessed {
        Ok(d) => d,
        Err(err) => {
            error!("Failed to get file {} date: {err}", path.display());
            return false;
        }
    };

    let res = &from_date > &duration;
    match res {
        true => debug!("File {} is older than {duration:?} ({from_date:?})", path.display()),
        false => debug!("File {} is newer than {duration:?} ({from_date:?})", path.display()),
    }

    res
}
