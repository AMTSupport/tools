/*
 * Copyright (C) 2024. James Draycott me@racci.dev
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

pub mod cli;
pub mod org;
pub mod rules;
pub mod user;

use crate::config::backend::Backend;
use crate::config::runtime::Runtime;
use crate::sources::downloader::Downloader;
use crate::sources::exporter::Exporter;
use amt_lib::fs::normalise_path;
use amt_lib::pathed::Pathed;
use anyhow::{anyhow, Context, Result};
use const_format::formatcp;
use indicatif::{MultiProgress, ProgressBar};
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

    // TODO: Implement login
    pub async fn login() -> Result<Self> {
        Ok(Self {
            user: "".to_string(),
            org_id: "".to_string(),
            org_name: "".to_string(),
            session_id: "".to_string(),
        })
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

    async fn interactive(config: &Runtime) -> Result<Vec<Backend>> {
        use amt_lib::ui::cli::ui_inquire::STYLE;
        use cli::LoginStatus;
        use inquire::{Password, PasswordDisplayMode, Text};
        use org::Organisation;
        use tracing::{error, info, trace};

        use crate::sources::getter::CliGetter;

        let username = Text::new("BitWarden Username")
            .with_render_config(*STYLE)
            .with_help_message("The username to use to log into BitWarden")
            .with_placeholder("username")
            .prompt()
            .with_context(|| "Get username for bitwarden user.")?;

        let data_dir = BitWardenCore::base_dir(config)?.join(&username);

        let command = || -> Result<Command> {
            let mut cmd = BitWardenCore::base_command(config)?;
            cmd.env(BitWardenCore::BW_DIRECTORY, &data_dir);
            Ok(cmd)
        };

        let envs = [(BitWardenCore::BW_DIRECTORY, data_dir.to_str().unwrap())];
        let status = LoginStatus::_get(config, &envs, &[]).await?;

        if let LoginStatus::Authenticated(user) = status {
            // TODO -> Prompt to log out?
            error!("Already logged into BitWarden as {user}");
            error!("Please remove {} and try again.", data_dir.display());
            return Err(anyhow!("Already logged into BitWarden as {user}"));
        }

        trace!("Not logged into BitWarden, logging in.");

        let password = Password::new("Bitwarden Password")
            .with_render_config(*STYLE)
            .with_help_message("The password to use to log into BitWarden")
            .without_confirmation()
            .with_display_mode(PasswordDisplayMode::Masked)
            .prompt()
            .with_context(|| "Get password for bitwarden user.")?;

        let two_fa = Text::new("Bitwarden 2FA")
            .with_render_config(*STYLE)
            .with_help_message("The 2FA code to use to log into BitWarden")
            .with_placeholder("123456")
            .prompt()
            .with_context(|| "Get 2FA code for bitwarden user.")?;

        let output = command()?
            .args(["login", &username, &password])
            .args(["--code", &two_fa])
            .arg("--raw")
            .output()
            .with_context(|| "Login to bitwarden")?;

        if !output.status.success() {
            return Err(anyhow!(
                "Failed to log into BitWarden -> {}",
                String::from_utf8_lossy(&output.stdout)
            ));
        }

        let session_id = String::from_utf8(output.stdout)?;
        trace!("Successfully logged into BitWarden");

        let organisations = command()?
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
            0 => Err(anyhow!("Unable to find any possible organisations to extract from!"))?,
            1 => {
                info!("Only one organisation found, using {}.", organisations[0].name);
                vec![Backend::BitWarden(BitWardenCore {
                    user: username,
                    org_id: organisations[0].id.clone(),
                    org_name: organisations[0].name.clone(),
                    session_id,
                })]
            }
            _ => inquire::MultiSelect::new("Select which organisations you would like to use.", organisations)
                .with_render_config(*STYLE)
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
}
