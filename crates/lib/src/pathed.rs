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

use anyhow::{anyhow, Context, Result};
use std::path::PathBuf;
use crate::fs::normalise_path;

pub trait Pathed<T>
where
    T: Into<PathBuf> + Clone,
{
    /// The Common of this source, used for the base directory.
    const NAME: &'static str;

    /// Permissions for the base directory.
    const PERMISSIONS: u32 = 0o755;

    /// # Returns
    /// Returns a new path for this source within the root directory
    fn base_dir(from: &T) -> Result<PathBuf> {
        let root: PathBuf = from.clone().into();
        let path = normalise_path(root.join(Self::NAME));
        let path = ensure_directory_exists(path)?;
        ensure_permissions(path, Self::PERMISSIONS)
    }

    /// # Returns
    /// Unique directory for this source instance within the base directory.
    fn unique_dir(&self, from: &T) -> Result<PathBuf> {
        let base = Self::base_dir(from)?;
        let name = self.get_unique_name();
        let path = normalise_path(base.join(name));
        let path = ensure_directory_exists(path)?;
        ensure_permissions(path, Self::PERMISSIONS)
    }

    /// # Returns
    /// Unique name for this source instance.
    fn get_unique_name(&self) -> String;
}

fn ensure_directory_exists(buf: PathBuf) -> Result<PathBuf> {
    if buf.exists() {
        return match &buf.is_dir() {
            false => Err(anyhow!(
                "Path exists but is not a directory: {}",
                buf.display()
            )),
            true => Ok(buf),
        };
    }

    std::fs::create_dir_all(&buf)
        .with_context(|| format!("Failed to create directories for dir of: {}", buf.display()))?;

    Ok(buf)
}

#[cfg(unix)]
fn ensure_permissions(buf: PathBuf, permissions: u32) -> Result<PathBuf> {
    use std::os::unix::prelude::PermissionsExt;
    std::fs::set_permissions(&buf, std::fs::Permissions::from_mode(permissions)).with_context(
        || {
            format!(
                "Failed to set required permissions on directory: {}",
                buf.display()
            )
        },
    )?;

    Ok(buf)
}

// TODO: Windows permissions
#[cfg(windows)]
fn ensure_permissions(buf: PathBuf, _permissions: u32) -> Result<PathBuf> {
    Ok(buf)
}
