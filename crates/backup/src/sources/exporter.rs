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
use crate::config::runtime::RuntimeConfig;
use crate::sources::bitwarden::BitWardenCore;
use crate::sources::downloader::Downloader;
use crate::sources::op::core::OnePasswordCore;
use crate::sources::s3::S3Core;
use anyhow::Result;
use async_trait::async_trait;
use clap::ValueEnum;
use indicatif::{MultiProgress, ProgressBar};
use lib::pathed::Pathed;
use serde::{Deserialize, Serialize};
use std::fmt::{Debug, Display, Formatter};

#[async_trait]
pub trait Exporter: Pathed<RuntimeConfig> {
    /// Used to attempt to interactively interactive a new exporter.
    async fn interactive(config: &RuntimeConfig) -> Result<Vec<Backend>>;

    // TODO :: Maybe return a reference to file/files which were exported?
    /// This method will export the backup data into memory,
    /// and then write it to the backup directory.
    async fn export(
        &mut self,
        config: &RuntimeConfig,
        main_bar: &ProgressBar,
        progress_bar: &MultiProgress,
    ) -> Result<()>;
}

#[derive(Debug, Clone, Serialize, Deserialize, ValueEnum)]
pub enum ExporterSource {
    S3,
    BitWarden,
    OnePassword,
}

impl Display for ExporterSource {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::S3 => write!(f, "S3"),
            Self::BitWarden => write!(f, "BitWarden"),
            Self::OnePassword => write!(f, "1Password"),
        }
    }
}

impl ExporterSource {
    pub async fn create(&self, config: &RuntimeConfig) -> Result<Vec<Backend>> {
        match self {
            Self::S3 => S3Core::interactive(config).await,
            Self::BitWarden => {
                let bars = MultiProgress::new();
                let main_bar = bars.add(ProgressBar::new_spinner());
                main_bar.set_message("Setting up BitWarden CLI");
                BitWardenCore::download_cli(config, &main_bar, &bars).await?;
                main_bar.finish_and_clear();

                BitWardenCore::interactive(config).await
            }
            Self::OnePassword => {
                let bars = MultiProgress::new();
                let main_bar = bars.add(ProgressBar::new_spinner());
                main_bar.set_message("Setting up 1Password CLI");
                OnePasswordCore::download_cli(config, &main_bar, &bars).await?;
                main_bar.finish_and_clear();

                OnePasswordCore::interactive(config).await
            }
        }
    }
}
