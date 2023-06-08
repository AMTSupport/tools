use crate::config::AutoPrune;
use crate::sources::{CustomSerialisable, Downloader};
use anyhow::Result;
use lib::anyhow;
use lib::anyhow::anyhow;
use lib::simplelog::trace;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use crate::sources::auto_prune::Prune;

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct BitwardenCore {
    id: String,
    name: String,
}

impl Prune for BitwardenCore {
    fn files(&self, root_directory: &PathBuf) -> Vec<PathBuf> {
        let directory = root_directory.join("BitWarden");
        if !directory.exists() {
            return vec![];
        }

        std::fs::read_dir(directory)
            .unwrap()
            .filter_map(|entry| entry.ok().map(|entry| entry.path()))
            .collect()
    }
}

impl Downloader for BitwardenCore {
    fn download(&self, root_directory: &PathBuf, _: &AutoPrune) -> Result<()> {
        let child = std::process::Command::new("bw")
            .arg("--organization")
            .arg(&self.id)
            .arg("export")
            .arg("--output")
            .arg(root_directory.join("BitWarden").join(format!(
                "{}_{}.json",
                &self.name,
                chrono::Local::now().format("%Y-%m-%d")
            )))
            .spawn()?;

        let out = child.wait_with_output()?;
        if out.status.exit_ok().is_err() {
            return Err(anyhow!(
                "Failed to export BitWarden organisation {}: {}",
                &self.name,
                String::from_utf8(out.stderr)?
            ));
        }

        trace!(
            "Successfully exported BitWarden organisation {}",
            &self.name
        );
        return Ok(());
    }
}

impl CustomSerialisable for BitwardenCore {
    fn serialisable(&self) -> serde_json::Value {
        serde_json::json!({
            "id": self.id,
            "name": self.name,
        })
    }
}

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
    let organisations: Vec<Organisation> = serde_json::from_str(&output)
        .expect("Failed to parse bw list organizations output as JSON");
    organisations
}
