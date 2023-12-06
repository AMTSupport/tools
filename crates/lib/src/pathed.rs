/*
 * Copyright (c) 2023. James Draycott <me@racci.dev>
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, version 3.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License along with this program.
 * If not, see <https://www.gnu.org/licenses/>.
 */

use crate::fs::normalise_path;
use anyhow::{anyhow, Context, Result};
use std::path::{Path, PathBuf};

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
        ensure_directory_exists(&path)?;
        ensure_permissions(&path, Self::PERMISSIONS)?;
        Ok(path)
    }

    /// # Returns
    /// Unique directory for this source instance within the base directory.
    fn unique_dir(&self, from: &T) -> Result<PathBuf> {
        let base = Self::base_dir(from)?;
        let name = self.get_unique_name();
        let path = normalise_path(base.join(name));
        ensure_directory_exists(&path)?;
        ensure_permissions(&path, Self::PERMISSIONS)?;
        Ok(path)
    }

    /// # Returns
    /// Unique name for this source instance.
    fn get_unique_name(&self) -> String;
}

pub fn ensure_directory_exists(path: &Path) -> Result<()> {
    if path.exists() {
        return match &path.is_dir() {
            true => Ok(()),
            false => Err(anyhow!("Path exists but is not a directory: {}", path.display())),
        };
    }

    std::fs::create_dir_all(path)
        .with_context(|| format!("Failed to create directories for dir of: {}", path.display()))?;

    Ok(())
}

#[cfg(unix)]
pub fn ensure_permissions(path: &Path, permissions: u32) -> Result<()> {
    use std::os::unix::prelude::PermissionsExt;
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(permissions))
        .with_context(|| format!("Failed to set required permissions on directory: {}", path.display()))?;

    Ok(())
}

// TODO: Windows permissions
#[cfg(windows)]
pub fn ensure_permissions(_path: &Path, _permissions: u32) -> Result<()> {
    tracing::warn!("Permissions are not supported on Windows yet, there may be issues.");
    Ok(())
}
