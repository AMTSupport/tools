use crate::config::backend::Backend;
use crate::config::runtime::RuntimeConfig;
use crate::sources::bitwarden::BitWardenCore;
use crate::sources::s3::S3Core;
use async_trait::async_trait;
use clap::ValueEnum;
use lib::anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fmt::{Debug, Display, Formatter};

#[async_trait]
pub trait Exporter {
    /// Used to attempt to interactively interactive a new exporter.
    fn interactive(config: &RuntimeConfig) -> Result<Vec<Backend>>;

    /// This method will export the backup data into memory,
    /// and then write it to the backup directory.
    async fn export(&mut self, config: &RuntimeConfig) -> Result<()>;
}

#[derive(Debug, Clone, Serialize, Deserialize, ValueEnum)]
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
    pub fn create(&self, config: &RuntimeConfig) -> Result<Vec<Backend>> {
        let exporters = match self {
            Self::S3 => S3Core::interactive(config),
            Self::BitWarden => BitWardenCore::interactive(config),
        };

        exporters
    }
}
