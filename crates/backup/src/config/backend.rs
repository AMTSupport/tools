/*
 * Copyright (C) 2023-2024. James Draycott me@racci.dev
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, version 3.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.
 * See the GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program. If not, see https://www.gnu.org/licenses/.
 */

use crate::config::runtime::Runtime;
use crate::sources::auto_prune::Prune;
use crate::sources::downloader::Downloader;
use crate::sources::exporter::Exporter;
use anyhow::Result;
use indicatif::{MultiProgress, ProgressBar};
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

#[cfg(feature = "sources-bitwarden")]
use crate::sources::bitwarden::BitWardenCore;
#[cfg(feature = "sources-1password")]
use crate::sources::op::core::OnePasswordCore;
#[cfg(feature = "sources-s3")]
use crate::sources::s3::S3Core;

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Backend {
    #[cfg(feature = "sources-s3")]
    S3(S3Core),
    #[cfg(feature = "sources-bitwarden")]
    BitWarden(BitWardenCore),
    #[cfg(feature = "sources-1password")]
    OnePassword(OnePasswordCore),
}

impl Display for Backend {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            #[cfg(feature = "sources-s3")]
            Backend::S3(s3) => write!(f, "S3 ({}:{})", &s3.base.get_bucket(), &s3.base.get_root().display()),
            #[cfg(feature = "sources-bitwarden")]
            Backend::BitWarden(bw) => write!(f, "BitWarden ({})", &bw.org_name),
            #[cfg(feature = "sources-1password")]
            Backend::OnePassword(op) => write!(f, "1Password ({})", &op.account),
        }
    }
}

impl Backend {
    pub async fn run(
        mut self,
        config: &Runtime,
        main_bar: &ProgressBar,
        progress_bar: &MultiProgress,
    ) -> Result<Backend> {
        match self {
            #[cfg(feature = "sources-s3")]
            Backend::S3(ref mut core) => {
                core.prune(config, progress_bar)?;
                core.export(config, main_bar, progress_bar).await?;
            }
            #[cfg(feature = "sources-bitwarden")]
            Backend::BitWarden(ref mut core) => {
                BitWardenCore::download_cli(config, main_bar, progress_bar).await?;
                core.prune(config, progress_bar)?;
                core.export(config, main_bar, progress_bar).await?;
            }
            #[cfg(feature = "sources-1password")]
            Backend::OnePassword(ref mut core) => {
                OnePasswordCore::download_cli(config, main_bar, progress_bar).await?;
                core.prune(config, progress_bar)?;
                core.export(config, main_bar, progress_bar).await?;
            }
        }

        Ok(self)
    }
}
