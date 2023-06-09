use crate::config::{AutoPrune, Backend};
use crate::sources::bitwarden::BitWardenCore;
use crate::sources::s3::S3Core;
use clap::ValueEnum;
use lib::anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fmt::{Debug, Display, Formatter};
use std::path::PathBuf;
use async_trait::async_trait;

#[async_trait]
pub trait Exporter {
    /// Used to attempt to interactively create a new exporter.
    fn create(interactive: bool) -> Result<Vec<Backend>>;

    /// This method will export the backup data into memory,
    /// and then write it to the backup directory.
    async fn export(&mut self, root_directory: &PathBuf, auto_prune: &AutoPrune) -> Result<()>;
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, ValueEnum)]
pub enum ExporterSource {
    S3,
    BitWarden,
}

impl Display for ExporterSource {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::S3 => write!(f, "S3"),
            Self::BitWarden => write!(f, "BitWarden"),
        }
    }
}

impl ExporterSource {
    pub fn create(&self, interactive: &bool) -> Result<Vec<Backend>> {
        let exporters = match self {
            Self::S3 => S3Core::create(interactive.clone()),
            Self::BitWarden => BitWardenCore::create(interactive.clone()),
        };

        exporters
    }
}
