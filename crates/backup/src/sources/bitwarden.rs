use crate::config::{AutoPrune, Backend, Config, RuntimeConfig};
use crate::sources::auto_prune::Prune;
use crate::sources::exporter::Exporter;
use crate::{continue_loop, env_or_prompt};
use anyhow::Result;
use async_trait::async_trait;
use core::slice::SlicePattern;
use futures::StreamExt;
use inquire::validator::Validation;
use lib::anyhow;
use lib::anyhow::{anyhow, Context};
use lib::simplelog::{debug, error, info, trace};
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::path::PathBuf;
use std::process::Command;
use std::string::ToString;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BitWardenCore {
    user: String,
    org_id: String,
    org_name: String,
    session_id: String,
}

impl BitWardenCore {
    const BW_SESSION: &'static str = "BW_SESSION";
    const BW_DIRECTORY: &'static str = "BITWARDENCLI_APPDATA_DIR";

    #[inline]
    fn base_dir(config: &RuntimeConfig) -> PathBuf {
        config.directory.join("BitWarden")
    }

    #[inline]
    fn data_dir(&self, config: &RuntimeConfig) -> PathBuf {
        Self::_data_dir(&config, &self.user)
    }

    #[inline]
    fn backup_dir(&self, config: &RuntimeConfig) -> PathBuf {
        Self::_backup_dir(&config, &self.org_name)
    }

    fn _data_dir(config: &RuntimeConfig, user: &str) -> PathBuf {
        Self::base_dir(&config).join(PathBuf::from(format!(r"data-{user}")))
    }

    fn _backup_dir(config: &RuntimeConfig, org_name: &str) -> PathBuf {
        Self::base_dir(&config).join(PathBuf::from(format!(r"backup-{org_name}")))
    }
}

impl Prune for BitWardenCore {
    fn files(&self, config: &RuntimeConfig) -> Vec<PathBuf> {
        let glob = glob::glob(&format!(
            "{root}/backup-{org}/*.json",
            root = &config.directory.display(),
            org = &self.org_name
        ))
        .unwrap(); // TODO: Handle this better.

        glob.filter_map(|entry| entry.ok()).collect()
    }
}

#[async_trait]
impl Exporter for BitWardenCore {
    fn interactive(config: &RuntimeConfig) -> Result<Vec<Backend>> {
        let username = inquire::Text::new("BitWarden Username").prompt()?;
        let data_dir = Self::_data_dir(&config, &username);
        let login_status = serde_json::from_slice::<LoginStatus>(
            cli(&data_dir).arg("status").output()?.stdout.as_slice(),
        )
        .context("Parse BitWarden status")?;

        let session_id = if login_status.status == "unauthenticated" {
            info!("Not logged into BitWarden, logging in.");

            let password = inquire::Text::new("BitWarden Password").prompt()?;
            let two_fa = inquire::Text::new("BitWarden 2FA").prompt()?;
            let cmd = cli(&data_dir)
                .arg("login")
                .arg(&username)
                .arg(password)
                .arg("--code")
                .arg(two_fa)
                .arg("--raw")
                .output()?;

            match cmd {
                out if out.status.success() => {
                    info!("Successfully logged into BitWarden");
                    String::from_utf8(out.stdout)?
                }
                out => {
                    info!("Failed to log into BitWarden");
                    return Err(anyhow!("Failed to log into BitWarden"));
                }
            }
        } else {
            // TODO: Support already logged in.
            error!("Already logged into BitWarden, but not supported yet.");
            error!(
                "Please remove the existing session file at {}, and try again.",
                &data_dir.display()
            );
            return Err(anyhow!("Already logged into BitWarden"));
        };

        let organisations = cli(&data_dir)
            .arg("list")
            .arg("organizations")
            .arg("--session")
            .arg(&session_id)
            .output()?
            .stdout;

        let organisations = serde_json::from_slice::<Vec<Organisation>>(organisations.as_slice())
            .context("Parse possible organisations")?;

        let organisations = match organisations.len() {
            0 => Err(anyhow!(
                "Unable to find any possible organisations to extract from!"
            ))?,
            len if len == 1 => {
                info!("Only one organisation found, using that one.");
                vec![Backend::BitWarden(BitWardenCore {
                    user: username.clone(),
                    org_id: organisations[0].id.clone(),
                    org_name: organisations[0].name.clone(),
                    session_id,
                })]
            }
            len => inquire::MultiSelect::new(
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

        info!("{:?}", &organisations);
        Ok(organisations)
    }

    async fn export(&mut self, config: &RuntimeConfig) -> Result<()> {
        let output_file = self.backup_dir(&config).join(format!(
            "{org_id}_{date}.json",
            org_id = &self.org_id,
            date = chrono::Local::now().format("%Y-%m-%d")
        ));

        let mut cmd = cli(&self.data_dir(&config))
            .env(Self::BW_SESSION, &self.session_id)
            .arg("export")
            .args(["--organizationid", &self.org_id])
            .args(["--format", "csv"])
            .args(["--output", output_file.to_str().unwrap()])
            .spawn()?;

        trace!("Running BitWarden export command: {:?}", &cmd);
        let status = cmd.wait()?;
        trace!("BitWarden export command finished: {:?}", &status);

        match status {
            s if s.success() => info!(
                "Successfully exported BitWarden organisation {name}",
                name = &self.org_name
            ),
            s => info!("BitWarden export command exited with code {:?}", &s.code()),
        }

        return Ok(());
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

fn cli(dir: &PathBuf) -> Command {
    let mut command = Command::new("bw");
    command.env(BitWardenCore::BW_DIRECTORY, dir.as_os_str());
    command
}
