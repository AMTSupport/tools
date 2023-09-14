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
use crate::config::backend::Backend::OnePassword;
use crate::config::runtime::Runtime;
use crate::sources::auto_prune::Prune;
use crate::sources::downloader::Downloader;
use crate::sources::exporter::Exporter;
use crate::sources::op::account::OnePasswordAccount;
use crate::sources::op::one_pux;
use anyhow::{anyhow, Context, Result};
use const_format::formatcp;
use indicatif::{MultiProgress, ProgressBar};
use lib::fs::normalise_path;
use lib::pathed::{ensure_directory_exists, ensure_permissions, Pathed};
use serde::{Deserialize, Serialize};
use serde_json::to_string_pretty;
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;
use std::{env, fs};
use zip::write::FileOptions;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OnePasswordCore {
    pub account: OnePasswordAccount,
}

impl OnePasswordCore {
    pub fn data_dir(config: &Runtime) -> Result<PathBuf> {
        let path = Self::base_dir(config)?.join("data");
        ensure_directory_exists(&path)?;
        ensure_permissions(&path, Self::PERMISSIONS)?;
        Ok(path)
    }
}

impl Pathed<Runtime> for OnePasswordCore {
    const NAME: &'static str = "1Password";

    fn get_unique_name(&self) -> String {
        self.account.get_unique_name()
    }
}

impl Downloader for OnePasswordCore {
    const BINARY: &'static str = if cfg!(windows) { "op.exe" } else { "op" };
    const URL: &'static str = formatcp!(
        "https://cache.agilebits.com/dist/1P/op2/pkg/{version}/op_{os}_{arch}_{version}.zip",
        version = "v2.18.0",
        os = env::consts::OS,
        arch = if cfg!(target_arch = "x86") {
            "386"
        } else if cfg!(target_arch = "x86_64") {
            "amd64"
        } else {
            panic!("Unsupported arch")
        }
    );

    fn base_command(config: &Runtime) -> Result<Command> {
        let mut command = Command::new(Self::binary(config)?);
        command.arg("--cache").args(["--config", Self::data_dir(config)?.to_str().context("Convert path to &str")?]);

        Ok(command)
    }
}

impl Exporter for OnePasswordCore {
    // TODO :: Export of extra stuff like logos in the zip
    // TODO :: I'm unsure if that's even possible though.
    /// Creates a 1PUX compatible export,
    ///
    /// The name of this file is in format of "1Password-{uuid of the account exporting it}-{%Y%m%d-%H%M%S}.1pux"
    async fn export(
        &mut self,
        runtime: &Runtime,
        main_bar: &ProgressBar,
        progress_bar: &MultiProgress,
    ) -> Result<()> {
        use chrono::Local;
        use one_pux::{attributes::Attributes, export::Export};

        let account = &self.account;
        let file_name = format!("1PasswordExport-{}.1pux", Local::now().format("%Y%m%d-%H%M%S"));

        let file = self.account.unique_dir(runtime)?.join(file_name);
        let file = normalise_path(file);

        let file = fs::File::create_new(file).context("Create export file")?;
        let mut zip = zip::ZipWriter::new(file);

        let options = FileOptions::default();
        let attributes = Attributes::default();
        let serialised = to_string_pretty(&attributes).context("Serialise to 1PUX")?;
        zip.start_file("export.attributes", options).context("Start writer for attrs.")?;
        zip.write_all(serialised.as_bytes()).context("Write attrs to zip file.")?;

        let (export, errors) = match Export::from(account, runtime, (main_bar, progress_bar)).await {
            Err(e) => {
                zip.finish().context("Finish export file")?;
                return Err(e);
            }
            Ok((export, errors)) => (export, errors),
        };

        let serialised = to_string_pretty(&export.data).context("Serialise to 1PUX")?;
        zip.start_file("export.data", options)?;
        zip.write_all(serialised.as_bytes())?;

        zip.add_directory("files", options).context("Create file directory")?;
        for file in export.files {
            zip.start_file(format!("files/{}", file.name), options)?;
            zip.write_all(&file.data)?;
        }

        zip.finish().context("Finish export file")?;

        if errors.len() > 0 {
            return Err(anyhow!("Errors occurred during export: {:?}", errors));
        }

        return Ok(());
    }
}

impl Prune for OnePasswordCore {
    fn files(&self, config: &Runtime) -> Result<Vec<PathBuf>> {
        use std::path::MAIN_SEPARATOR;

        let glob = format!(
            "{}{MAIN_SEPARATOR}export_*.zip",
            self.account.unique_dir(config)?.display()
        );

        glob::glob(&glob).with_context(|| format!("Glob for files in {}", glob)).map(|glob| glob.flatten().collect())
    }
}

#[cfg(feature = "ui-cli")]
pub(crate) async fn interactive(config: &Runtime) -> Result<Vec<Backend>> {
    use crate::sources::interactive::Interactive;

    let account = OnePasswordAccount::interactive(config).await?;
    Ok(vec![OnePassword(OnePasswordCore { account })])
}
