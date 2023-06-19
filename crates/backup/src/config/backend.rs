use crate::config::runtime::RuntimeConfig;
use crate::sources::auto_prune::Prune;
use crate::sources::bitwarden::BitWardenCore;
use crate::sources::downloader::Downloader;
use crate::sources::exporter::Exporter;
use crate::sources::op::core::OnePasswordCore;
use crate::sources::s3::S3Core;
use indicatif::{MultiProgress, ProgressBar};
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
    pub async fn run(
        mut self,
        config: &RuntimeConfig,
        main_bar: &ProgressBar,
        progress_bar: &MultiProgress,
    ) -> Result<Backend> {
        match self {
            Backend::S3(ref mut core) => {
                core.prune(&config, &progress_bar)?;
                core.export(&config, &main_bar, &progress_bar).await?;
            }
            Backend::BitWarden(ref mut core) => {
                BitWardenCore::download_cli(&config, &main_bar, &progress_bar).await?;
                core.prune(&config, &progress_bar)?;
                core.export(&config, &main_bar, &progress_bar).await?;
            }
            Backend::OnePassword(ref mut core) => {
                OnePasswordCore::download_cli(&config, &main_bar, &progress_bar).await?;
                core.prune(&config, &progress_bar)?;
                core.export(&config, &main_bar, &progress_bar).await?;
            }
        }

        Ok(self)
    }
}
