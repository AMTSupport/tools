use crate::config::backend::Backend;
use crate::config::runtime::RuntimeConfig;
use crate::sources::bitwarden::BitWardenCore;
use crate::sources::op::core::OnePasswordCore;
use crate::sources::s3::S3Core;
use async_trait::async_trait;
use clap::ValueEnum;
use lib::anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fmt::{Debug, Display, Formatter};

#[async_trait]
pub trait Exporter {
    /// Used to attempt to interactively interactive a new exporter.
    async fn interactive(config: &RuntimeConfig) -> Result<Vec<Backend>>;

    // TODO :: Maybe return a reference to file/files which were exported?
    /// This method will export the backup data into memory,
    /// and then write it to the backup directory.
    async fn export(&mut self, config: &RuntimeConfig) -> Result<()>;
}

#[derive(Debug, Clone, Serialize, Deserialize, ValueEnum)]
pub enum ExporterSource {
    S3,
    BitWarden,
    OnePassword,
}

impl Display for ExporterSource {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::S3 => write!(f, "S3"),
            Self::BitWarden => write!(f, "BitWarden"),
            Self::OnePassword => write!(f, "1Password"),
        }
    }
}

impl ExporterSource {
    pub async fn create(&self, config: &RuntimeConfig) -> Result<Vec<Backend>> {
        let exporters = match self {
            Self::S3 => S3Core::interactive(config).await,
            Self::BitWarden => BitWardenCore::interactive(config).await,
            Self::OnePassword => OnePasswordCore::interactive(config).await,
        };

        exporters
    }
}
