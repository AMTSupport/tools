use crate::config::runtime::RuntimeConfig;
use crate::sources::auto_prune::Prune;
use crate::sources::bitwarden::BitWardenCore;
use crate::sources::exporter::Exporter;
use crate::sources::op::core::OnePasswordCore;
use crate::sources::s3::S3Core;
use lib::anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Backend {
    S3(S3Core),
    BitWarden(BitWardenCore),
    OnePassword(OnePasswordCore),
}

impl Display for Backend {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Backend::S3(s3) => write!(
                f,
                "S3 ({}:{})",
                &s3.base.backend.get("bucket").unwrap(),
                &s3.base.object.display()
            ),
            Backend::BitWarden(bw) => write!(f, "BitWarden ({})", &bw.org_name),
            Backend::OnePassword(op) => write!(f, "1Password ({})", &op.account),
        }
    }
}

impl Backend {
    pub async fn run(&mut self, config: &RuntimeConfig) -> Result<()> {
        match self {
            Backend::S3(ref mut core) => {
                core.prune(&config)?;
                core.export(&config).await?;
            }
            Backend::BitWarden(ref mut core) => {
                core.prune(&config)?;
                core.export(&config).await?;
            }
            Backend::OnePassword(ref mut core) => {
                core.prune(&config)?;
                core.export(&config).await?;
            }
        }

        Ok(())
    }
}
