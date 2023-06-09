use crate::config::{AutoPrune, Backend, RuntimeConfig};
use crate::sources::auto_prune::Prune;
use crate::sources::exporter::Exporter;
use crate::{continue_loop, env_or_prompt};
use anyhow::Result;
use async_trait::async_trait;
use inquire::validator::Validation;
use lib::anyhow;
use lib::anyhow::anyhow;
use lib::simplelog::{info, trace};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct BitWardenCore {
    org_id: String,
    org_name: String,
    session_id: String,
}

impl Prune for BitWardenCore {
    fn files(&self, config: &RuntimeConfig) -> Vec<PathBuf> {
        let directory = config.directory.join("BitWarden");
        if !directory.exists() {
            return vec![];
        }

        std::fs::read_dir(directory)
            .unwrap()
            .filter_map(|entry| entry.ok().map(|entry| entry.path()))
            .collect()
    }
}

#[async_trait]
impl Exporter for BitWardenCore {
    fn interactive(config: &RuntimeConfig) -> Result<Vec<Backend>> {
        let bw = std::process::Command::new("bw")
            .env("BITWARDENCLI_APPDATA_DIR", config.directory.join("BitWarden/data"))
            .spawn();

        info!("{:?}", bw);

        // let client_id = env_or_prompt("BW_CLIENTID", &interactive, move |str: &_| {
        //     match str.chars().any(|c| !c.is_ascii_alphanumeric() || c == '.' || c == '-') {
        //         false => Ok(Validation::Valid),
        //         true => Ok(Validation::Invalid(
        //             "Client ID must be only alphanumeric characters, '.', and '-'".into(),
        //         )),
        //     }
        // })?;
        //
        // let client_secret = env_or_prompt("BW_CLIENTSECRET", &interactive, move |str: &_| {
        //     match str.chars().any(|c| !c.is_ascii_alphanumeric()) {
        //         false => Ok(Validation::Valid),
        //         true => Ok(Validation::Invalid(
        //             "Client Key must be only alphanumeric characters".into(),
        //         )),
        //     }
        // })?;



        Ok(vec![])
    }

    async fn export(&mut self, config: &RuntimeConfig) -> Result<()> {
        let child = std::process::Command::new("bw")
            .arg("export")
            .arg("--organizationid")
            .arg(&self.org_id)
            .arg("--format")
            .arg("json")
            .arg("--output")
            .arg(config.directory.join("BitWarden").join(format!(
                "{}_{}.json",
                &self.org_name,
                chrono::Local::now().format("%Y-%m-%d")
            )))
            .spawn()?;

        let out = child.wait_with_output()?;
        if out.status.exit_ok().is_err() {
            return Err(anyhow!(
                "Failed to export BitWarden organisation {}: {}",
                &self.org_name,
                String::from_utf8(out.stderr)?
            ));
        }

        trace!(
            "Successfully exported BitWarden organisation {}",
            &self.org_name
        );
        return Ok(());
    }
}

#[derive(Serialize, Deserialize)]
struct Organisation {
    id: String,
    name: String,
}

fn get_organisations() -> Vec<Organisation> {
    let child = std::process::Command::new("bw")
        .arg("list")
        .arg("organizations")
        .stdout(std::process::Stdio::piped())
        .spawn()
        .expect("Failed to spawn bw list organizations");

    let output = child
        .wait_with_output()
        .expect("Failed to wait for bw list organizations");
    let output = String::from_utf8(output.stdout)
        .expect("Failed to parse bw list organizations output as UTF-8");
    let mut organisations: Vec<Organisation> = serde_json::from_str(&output)
        .expect("Failed to parse bw list organizations output as JSON");

    organisations.sort_by(|a, b| a.name.cmp(&b.name));
    organisations
}
