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
use crate::sources::auto_prune::Prune;
use crate::sources::downloader::Downloader;
use crate::sources::exporter::Exporter;
use anyhow::Result;
use async_trait::async_trait;
use const_format::formatcp;
use indicatif::{MultiProgress, ProgressBar};
use lib::anyhow;
use lib::anyhow::{anyhow, Context};
use lib::fs::normalise_path;
use serde::{Deserialize, Serialize};
use std::env;
use std::fmt::{Display, Formatter};
use std::path::PathBuf;
use std::process::Command;
use tracing::{error, info};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BitWardenCore {
    pub user: String,
    pub org_id: String,
    pub org_name: String,
    session_id: String,
}

impl BitWardenCore {
    const BW_SESSION: &'static str = "BW_SESSION";
    const BW_DIRECTORY: &'static str = "BITWARDENCLI_APPDATA_DIR";

    fn data_dir(&self, config: &RuntimeConfig) -> PathBuf {
        Self::_data_dir(config, &self.user)
    }

    fn backup_dir(&self, config: &RuntimeConfig) -> PathBuf {
        Self::_backup_dir(config, &self.org_name)
    }

    fn _data_dir(config: &RuntimeConfig, user: &str) -> PathBuf {
        Self::base_dir(config).join(PathBuf::from(format!(r"data-{user}")))
    }

    fn _backup_dir(config: &RuntimeConfig, org_name: &str) -> PathBuf {
        Self::base_dir(config).join(PathBuf::from(format!(r"backup-{org_name}")))
    }

    fn command(&self, config: &RuntimeConfig) -> Command {
        let mut cmd = Self::base_command(config);
        cmd.env(Self::BW_DIRECTORY, &self.data_dir(config));
        cmd.env(Self::BW_SESSION, &self.session_id);
        cmd
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

impl Prune for BitWardenCore {
    fn files(&self, config: &RuntimeConfig) -> Result<Vec<PathBuf>> {
        use std::path::MAIN_SEPARATOR;

        let glob = format!(
            "{root}{MAIN_SEPARATOR}backup-{org}/*.json",
            root = &config.directory.display(),
            org = &self.org_name
        );

        glob::glob(&glob)
            .with_context(|| format!("Glob backup files for {glob}"))
            .map(|g| g.flatten().collect())
    }
}

#[async_trait]
impl Exporter for BitWardenCore {
    const DIRECTORY: &'static str = "Bitwarden";

    async fn interactive(config: &RuntimeConfig) -> Result<Vec<Backend>> {
        let username = inquire::Text::new("BitWarden Username").prompt()?;
        let data_dir = Self::_data_dir(config, &username);

        let command = || -> Command {
            let mut cmd = BitWardenCore::base_command(config);
            cmd.env(Self::BW_DIRECTORY, &data_dir);
            cmd
        };

        let login_status = serde_json::from_slice::<LoginStatus>(
            command().arg("status").output().context("Get login status")?.stdout.as_slice(),
        )
        .context("Parse BitWarden status")?;

        let session_id = if login_status.status == "unauthenticated" {
            info!("Not logged into BitWarden, logging in.");

            let password = inquire::Text::new("BitWarden Password").prompt()?;
            let two_fa = inquire::Text::new("BitWarden 2FA").prompt()?;
            let cmd = command()
                .arg("login")
                .arg(&username)
                .arg(password)
                .arg("--code")
                .arg(two_fa)
                .arg("--raw")
                .output()
                .context("Login to bitwarden")?;

            match cmd {
                out if out.status.success() => {
                    info!("Successfully logged into BitWarden");
                    String::from_utf8(out.stdout)?
                }
                _ => {
                    info!("Failed to log into BitWarden");
                    return Err(anyhow!("Failed to log into BitWarden"));
                }
            }
        } else {
            // TODO: Support already logged in.
            // TODO -> Prompt to log out?
            error!("Already logged into BitWarden, but not supported yet.");
            error!(
                "Please remove the existing session file at {}, and try again.",
                &data_dir.display()
            );
            return Err(anyhow!("Already logged into BitWarden"));
        };

        let organisations = command()
            .arg("list")
            .arg("organizations")
            .arg("--session")
            .arg(&session_id)
            .output()
            .context("Get possible organisations")?
            .stdout;

        let organisations = serde_json::from_slice::<Vec<Organisation>>(organisations.as_slice())
            .context("Parse possible organisations")?;

        let organisations = match organisations.len() {
            0 => Err(anyhow!(
                "Unable to find any possible organisations to extract from!"
            ))?,
            1 => {
                info!(
                    "Only one organisation found, using {}.",
                    organisations[0].name
                );
                vec![Backend::BitWarden(BitWardenCore {
                    user: username,
                    org_id: organisations[0].id.clone(),
                    org_name: organisations[0].name.clone(),
                    session_id,
                })]
            }
            _ => inquire::MultiSelect::new(
                "Select which organisations you would like to use.",
                organisations,
            )
            .prompt()?
            .iter()
            .map(|org| {
                Backend::BitWarden(BitWardenCore {
                    user: username.clone(),
                    org_id: org.id.clone(),
                    org_name: org.name.clone(),
                    session_id: session_id.clone(),
                })
            })
            .collect(),
        };

        Ok(organisations)
    }

    async fn export(
        &mut self,
        config: &RuntimeConfig,
        _main_bar: &ProgressBar,
        _progress_bar: &MultiProgress,
    ) -> Result<()> {
        let export = |format: &str, ext: &str| -> Result<()> {
            let output_file = normalise_path(self.backup_dir(config).join(format!(
                "{org_id}_{date}-{format}.{ext}",
                org_id = &self.org_id,
                date = chrono::Local::now().format("%Y-%m-%dT%H:%M:%SZ%z")
            )));

            let cmd = self
                .command(config)
                .arg("export")
                .args(["--organizationid", &self.org_id])
                .args(["--format", format])
                .args(["--output", output_file.to_str().unwrap()])
                .output()
                .context(format!("Create bitwarden export for {}", &self.org_name))?;

            if !cmd.stderr.is_empty() {
                let string = String::from_utf8(cmd.stderr)?;
                return Err(anyhow!(
                    "BitWarden export for {} failed: {string}",
                    &self.org_name
                ));
            }

            Ok(())
        };

        export("encrypted_json", "json")?;
        export("json", "json")?;
        export("csv", "csv")
    }
}

#[derive(Serialize, Deserialize)]
struct LoginStatus {
    #[serde(rename = "userEmail", default = "String::new")]
    user_email: String,
    status: String,
}

#[derive(Serialize, Deserialize)]
struct Organisation {
    id: String,
    name: String,
}

impl Display for Organisation {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ({})", self.name, self.id)
    }
}
