/*
 * Copyright (c) 2023-2024. James Draycott <me@racci.dev>
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
 * You should have received a copy of the GNU General Public License along with this program.
 * If not, see <https://www.gnu.org/licenses/>.
 */

pub mod cli;
pub mod org;
pub mod rules;
pub mod user;

use crate::config::runtime::Runtime;
use crate::sources::downloader::Downloader;
use crate::sources::exporter::Exporter;
use anyhow::{anyhow, Context, Result};
use const_format::formatcp;
use indicatif::{MultiProgress, ProgressBar};
use lib::builder;
use lib::fs::normalise_path;
use lib::pathed::Pathed;
use std::env;
use std::fmt::{Display, Formatter};
use std::process::Command;

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct BitWardenCore {
    pub user: String,
    pub org_id: String,
    pub org_name: String,
    session_id: String,
}

impl BitWardenCore {
    const BW_SESSION: &'static str = "BW_SESSION";
    const BW_DIRECTORY: &'static str = "BITWARDENCLI_APPDATA_DIR";

    pub async fn login() -> Result<Self> {
        use macros::Builder;
    }

    fn command(&self, config: &Runtime) -> Result<Command> {
        let mut cmd = Self::base_command(config)?;
        cmd.env(Self::BW_DIRECTORY, &self.unique_dir(config)?);
        cmd.env(Self::BW_SESSION, &self.session_id);

        Ok(cmd)
    }
}

impl Display for BitWardenCore {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}/{}", self.user, self.org_name)
    }
}

impl Pathed<Runtime> for BitWardenCore {
    const NAME: &'static str = "Bitwarden";

    fn get_unique_name(&self) -> String {
        self.to_string()
    }
}

impl Downloader for BitWardenCore {
    const BINARY: &'static str = if cfg!(windows) { "bw.exe" } else { "bw" };
    const URL: &'static str = formatcp!(
        "https://github.com/bitwarden/clients/releases/download/cli-v{version}/bw-{os}-{version}.zip",
        version = "2023.5.0",
        os = env::consts::OS,
    );
}

impl Exporter for BitWardenCore {
    async fn export(
        &mut self,
        runtime: &Runtime,
        _main_bar: &ProgressBar,
        _progress_bar: &MultiProgress,
    ) -> Result<()> {
        let export = |format: &str, ext: &str| -> Result<()> {
            let output_file = normalise_path(self.unique_dir(runtime)?.join(format!(
                "{org_id}_{date}-{format}.{ext}",
                org_id = &self.org_id,
                date = chrono::Local::now().format("%Y-%m-%dT%H:%M:%SZ%z")
            )));

            let cmd = self
                .command(runtime)?
                .arg("export")
                .args(["--organizationid", &self.org_id])
                .args(["--format", format])
                .args(["--output", output_file.to_str().unwrap()])
                .output()
                .with_context(|| format!("Create bitwarden export for {}", &self.org_name))?;

            if !cmd.stderr.is_empty() {
                let string = String::from_utf8(cmd.stderr)?;
                return Err(anyhow!("BitWarden export for {} failed: {string}", &self.org_name));
            }

            Ok(())
        };

        export("encrypted_json", "json")?;
        export("json", "json")?;
        export("csv", "csv")
    }
}
