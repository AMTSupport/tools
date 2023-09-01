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

use crate::config::runtime::Runtime;
use crate::sources::download;
use crate::sources::exporter::Exporter;
use anyhow::{anyhow, Context, Result};
use async_trait::async_trait;
use futures_util::StreamExt;
use indicatif::{MultiProgress, ProgressBar};
use lib::fs::create_parents;
use lib::progress;
use std::fs::File;
use std::io::copy;
use std::path::PathBuf;
use std::process::Command;
use tracing::{debug, trace};

#[async_trait]
pub trait Downloader: Exporter {
    const BINARY: &'static str;
    const URL: &'static str;

    fn binary(config: &Runtime) -> Result<PathBuf> {
        Ok(Self::base_dir(config)?.join(Self::BINARY))
    }

    fn base_command(config: &Runtime) -> Result<Command> {
        Ok(Command::new(Self::binary(config)?))
    }

    async fn download_cli(
        config: &Runtime,
        main_bar: &ProgressBar,
        multi_bar: &MultiProgress,
    ) -> Result<()> {
        let target = Self::binary(config)?;
        create_parents(&target)?;

        // TODO :: Check for correct version, platform & arch
        if target.exists() && target.is_file() {
            debug!("Using existing CLI binary: {}", &target.display());
            return Ok(());
        }

        debug!("Downloading CLI binary from {} to {}", Self::URL, &target.display());
        let response = reqwest::Client::new().get(Self::URL).send().await?;
        if !response.status().is_success() {
            return Err(anyhow!("Failed to download CLI: {}", response.status()));
        }

        let total_size = response.content_length().unwrap();
        let stream = response.bytes_stream().boxed();

        let download_bar = multi_bar.insert_after(main_bar, progress::download());
        let download = download(total_size, stream, &download_bar).await?;
        download_bar.finish_and_clear();

        let file = File::open(&download).context("Open Download File")?;
        let mut archive = zip::ZipArchive::new(file).context("Open Zip Archive")?;
        let mut found = false;

        debug!("Searching for CLI binary in archive");
        debug!("Archive contains {} files", archive.len());
        debug!(
            "Archive contains {}",
            archive.file_names().collect::<Vec<_>>().join(", ")
        );
        // TODO :: Generic function for finding file in archive
        for i in 0..archive.len() {
            trace!("Checking file {}", i);
            let mut file = archive.by_index(i)?;
            debug!("Checking file {}", file.name());
            if file.is_file() && file.name() == target.file_name().unwrap() {
                create_parents(&target)?;
                let mut out = File::create(&target).context("Create file for CLI binary")?;
                copy(&mut file, &mut out).context("Copy CLI binary from archive to file")?;

                found = true;
                break;
            }
        }

        if !found {
            return Err(anyhow::anyhow!("Failed to find CLI binary in archive"));
        }

        Ok(())
    }
}
