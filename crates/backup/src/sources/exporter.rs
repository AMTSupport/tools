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

use crate::config::backend::Backend;
use crate::config::runtime::Runtime;
use crate::sources::bitwarden::BitWardenCore;
use crate::sources::downloader::Downloader;
use crate::sources::op::core::OnePasswordCore;
use anyhow::Result;
use async_trait::async_trait;
use indicatif::{MultiProgress, ProgressBar};
use lib::pathed::Pathed;
use macros::{EnumNames, EnumVariants};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use crate::sources::{bitwarden, op, s3};

#[async_trait]
pub trait Exporter: Pathed<Runtime> {
    /// Used to attempt to interactively interactive a new exporter.
    // async fn interactive(config: &Runtime) -> Result<Vec<Backend>>;

    // TODO :: Maybe return a reference to file/files which were exported?
    /// This method will export the backup data into memory,
    /// and then write it to the backup directory.
    async fn export(
        &mut self,
        runtime: &Runtime,
        main_bar: &ProgressBar,
        progress_bar: &MultiProgress,
    ) -> Result<()>;
}

#[derive(Debug, Clone, Serialize, Deserialize, EnumVariants, EnumNames)]
pub enum ExporterSource {
    S3,
    BitWarden,
    OnePassword,
}

impl ExporterSource {
    pub async fn create(&self, runtime: &Runtime) -> Result<Vec<Backend>> {
        match self {
            Self::S3 => s3::interactive(runtime).await,
            Self::BitWarden => {
                let bars = MultiProgress::new();
                let main_bar = bars.add(ProgressBar::new_spinner());
                main_bar.set_message("Setting up BitWarden CLI");
                BitWardenCore::download_cli(runtime, &main_bar, &bars).await?;
                main_bar.finish_and_clear();

                #[cfg(feature = "ui-cli")]
                bitwarden::interactive(runtime).await
            }
            Self::OnePassword => {
                let bars = MultiProgress::new();
                let main_bar = bars.add(ProgressBar::new_spinner());
                main_bar.set_message("Setting up 1Password CLI");
                OnePasswordCore::download_cli(runtime, &main_bar, &bars).await?;
                main_bar.finish_and_clear();

                op::core::interactive(runtime).await
            }
        }
    }
}
