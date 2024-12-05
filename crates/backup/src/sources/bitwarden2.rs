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

use crate::config::backend::Backend;
use crate::config::runtime::Runtime;
use crate::sources::auto_prune::Prune;
use crate::sources::downloader::Downloader;
use crate::sources::exporter::Exporter;
use crate::sources::getter::CliGetter;
use anyhow::{anyhow, Context, Result};
use const_format::formatcp;
use indicatif::{MultiProgress, ProgressBar};
use inquire::PasswordDisplayMode;
use amt_lib::fs::normalise_path;
use amt_lib::pathed::Pathed;
use serde::{Deserialize, Serialize};
use std::env;
use std::fmt::{Display, Formatter};
use std::path::PathBuf;
use std::process::Command;
use tracing::{error, info, trace};

#[cfg(feature = "ui-cli")]
pub(crate) async fn interactive(config: &Runtime) -> Result<Vec<Backend>> {
    use inquire::{Password, Text};
    use amt
    -lib::ui::cli::ui_inquire::inquire_style;

    let username = Text::new("BitWarden Username")
        .with_render_config(inquire_style())
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
        .with_render_config(inquire_style())
        .with_help_message("The password to use to log into BitWarden")
        .without_confirmation()
        .with_display_mode(PasswordDisplayMode::Masked)
        .prompt()
        .with_context(|| "Get password for bitwarden user.")?;

    let two_fa = Text::new("Bitwarden 2FA")
        .with_render_config(inquire_style())
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
