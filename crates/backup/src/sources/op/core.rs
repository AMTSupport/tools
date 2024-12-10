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

use crate::config::backend::Backend;
use crate::config::backend::Backend::OnePassword;
use crate::config::runtime::Runtime;
use crate::sources::auto_prune::Prune;
use crate::sources::downloader::Downloader;
use crate::sources::exporter::Exporter;
use crate::sources::getter::CliGetter;
use crate::sources::op::account::{AccountAttrs, OnePasswordAccount};
use crate::sources::op::one_pux;
use amt_lib::fs::normalise_path;
use amt_lib::pathed::{ensure_directory_exists, ensure_permissions, Pathed};
use amt_lib::ui::cli::ui_inquire::STYLE;
use anyhow::{anyhow, Context, Result};
use const_format::formatcp;
use futures_util::TryFutureExt;
use indicatif::{MultiProgress, ProgressBar};
use serde::{Deserialize, Serialize};
use serde_json::to_string_pretty;
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;
use std::{env, fs};
use tracing::{trace, warn};
use zip::write::SimpleFileOptions;

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
    /// The name of the 1Password CLI binary.
    const BINARY: &'static str = formatcp!("op{ext}", ext = env::consts::EXE_SUFFIX);

    /// The URL to download the 1Password CLI binary from.
    ///
    /// These URLs are generated based on the downloads from https://app-updates.agilebits.com/product_history/CLI2.
    const URL: &'static str = formatcp!(
        "https://cache.agilebits.com/dist/1P/op2/pkg/{version}/op_{os}_{arch}_{version}.zip",
        version = "v2.18.0",
        os = if cfg!(target_os = "macos") {
            "darwin"
        } else {
            env::consts::OS
        },
        arch = if cfg!(target_arch = "x86") {
            "386"
        } else if cfg!(target_arch = "x86_64") {
            "amd64"
        } else if cfg!(target_arch = "arm") {
            "arm"
        } else if cfg!(target_arch = "aarch64") {
            "arm64"
        } else {
            panic!("Unsupported architecture")
        }
    );

    fn base_command(config: &Runtime) -> Result<Command> {
        let mut command = Command::new(Self::binary(config)?);
        command
            .arg("--cache")
            .args(["--config", Self::data_dir(config)?.to_str().context("Convert path to &str")?]);

        Ok(command)
    }
}

impl Exporter for OnePasswordCore {
    async fn interactive(config: &Runtime) -> Result<Vec<Backend>> {
        use inquire::{list_option::ListOption, validator::Validation, MultiSelect, Select, Text};
        let selection = Select::new(
            "What type of Account do you want to setup.",
            vec!["Personal", "Service"],
        ).with_render_config(*STYLE).with_help_message(r#"
            A Service Account is a special type of account which can be logged in with a single token, however it cannot access Personal Vaults.
            A Personal Account is the standard way of authenticating with the cli which requires the 1Password desktop application to be installed,
            When using a Personal Account please ensure that the 1Password Desktop app doesn't have cli integration enabled.
        "#.trim()).prompt().with_context(|| "Prompt for account type")?;

        match selection {
            "Personal" => Err(anyhow!("Personal accounts are not yet supported.")),
            "Service" => {
                trace!("Prompting for service account token");
                // TODO :: Wrong url
                let token = Text::new("Enter your 1Password service token")
                    .with_help_message(
                        "You can get a service token at https://my.1password.com/integrations/infrastructure-secrets",
                    )
                    .with_validator(|t: &str| match t.len() {
                        0 => Ok(Validation::Invalid("Token cannot be empty".into())),
                        _ if !t.starts_with("ops_") => {
                            Ok(Validation::Invalid("Valid Service Token must start with 'ops_'".into()))
                        }
                        _ => Ok(Validation::Valid),
                    })
                    .with_placeholder("ops_...")
                    .prompt()
                    .map(|t| {
                        let t = t.trim();
                        t.to_owned()
                    })
                    .with_context(|| "Get service token input from user")?;

                use super::cli::{
                    account::{Account, AccountShort},
                    user::User,
                    vault::Reference,
                };

                let envs: [(&str, &str); 1] = [("OP_SERVICE_ACCOUNT_TOKEN", &token)];
                let user = User::_get(config, &envs, &[]);
                let short = AccountShort::_get(config, &envs, &[]);
                let account = Account::_get(config, &envs, &[]).and_then(|a| async move {
                    let attrs = a.attrs();
                    let short = match short
                        .await
                        .inspect_err(|e| warn!("Failed to get short account: {}", e))
                        .ok()
                        .and_then(|s| s.into_iter().find(|s| s.account_uuid == attrs.identifier.id()))
                    {
                        None => return Ok(a),
                        s => s,
                    };

                    Ok(match a {
                        Account::Business { attrs, .. } => Account::Business { attrs, short },
                        Account::Individual { attrs, .. } => Account::Individual { attrs, short },
                    })
                });

                let vaults = Reference::_get(config, &envs, &[]).and_then(|v| async {
                    match v.len() {
                        0 => Err(anyhow!("No vaults found for this account.")),
                        _ => MultiSelect::new("Select the vaults you want to use.", v)
                            .with_render_config(*STYLE)
                            .with_validator(|selections: &[ListOption<&Reference>]| match selections.len() {
                                0 => Ok(Validation::Invalid("You must select at least one vault.".into())),
                                _ => Ok(Validation::Valid),
                            })
                            .prompt()
                            .context("Get vaults from user selection"),
                    }
                });

                Ok(vec![OnePassword(OnePasswordCore {
                    account: OnePasswordAccount::Service {
                        attrs: AccountAttrs {
                            user: user.await?,
                            account: account.await?,
                            vaults: vaults.await?,
                        },
                        token,
                    },
                })])
            }
            _ => unreachable!("Invalid account type shouldn't be possible."),
        }
    }

    // TODO :: Export of extra stuff like logos in the zip
    // TODO :: I'm unsure if that's even possible though.
    /// Creates a 1PUX compatible export,
    ///
    /// The name of this file is in format of "1Password-{uuid of the account exporting it}-{%Y%m%d-%H%M%S}.1pux"
    async fn export(&mut self, runtime: &Runtime, main_bar: &ProgressBar, progress_bar: &MultiProgress) -> Result<()> {
        use chrono::Local;
        use one_pux::{attributes::Attributes, export::Export};

        let account = &self.account;
        let file_name = format!("1PasswordExport-{}.1pux", Local::now().format("%Y%m%d-%H%M%S"));

        let file = self.account.unique_dir(runtime)?.join(file_name);
        let file = normalise_path(file);

        let file = fs::File::create_new(file).context("Create export file")?;
        let mut zip = zip::ZipWriter::new(file);

        let options = SimpleFileOptions::default();
        let attributes = Attributes::default();
        let serialised = to_string_pretty(&attributes).context("Serialise to 1PUX")?;
        zip.start_file("export.attributes", options)
            .context("Start writer for attrs.")?;
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

        if !errors.is_empty() {
            return Err(anyhow!("Errors occurred during export: {:?}", errors));
        }

        Ok(())
    }
}

impl Prune for OnePasswordCore {
    fn files(&self, config: &Runtime) -> Result<Vec<PathBuf>> {
        use std::path::MAIN_SEPARATOR;

        let glob = format!(
            "{}{MAIN_SEPARATOR}export_*.zip",
            self.account.unique_dir(config)?.display()
        );

        glob::glob(&glob)
            .with_context(|| format!("Glob for files in {}", glob))
            .map(|glob| glob.flatten().collect())
    }
}
