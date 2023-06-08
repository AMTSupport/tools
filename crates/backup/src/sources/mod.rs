#![feature(trait_alias)]
use crate::config::AutoPrune;
use crate::continue_loop;
use crate::sources::auto_prune::Prune;
use crate::sources::s3::S3Core;
use async_trait::async_trait;
use clap::ValueEnum;
use lib::anyhow;
use lib::simplelog::info;
use opendal::Operator;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::{Debug, Display, Formatter};
use std::path::PathBuf;

pub mod auto_prune;
pub mod bitwarden;
pub mod s3;

#[derive(Debug, Clone, ValueEnum, Serialize, Deserialize, Eq, PartialEq)]
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
    pub fn create(&self, interactive: &bool) -> anyhow::Result<Vec<Box<dyn Backend>>> {
        let exporters = match self {
            Self::S3 => {
                let base = s3::create_operator(&interactive)?;
                let mut exporters = vec![];

                while continue_loop(&exporters, "object to export") {
                    match inquire::Text::new("What's the path of the object you want to export?")
                        .prompt()?
                    {
                        object_path if object_path.is_empty() => {
                            info!("Assuming wanted to cancel additional object.");
                            continue;
                        }
                        object_path => {
                            let mut base = base.base.clone();
                            base.backend.insert("root".to_string(), object_path.clone());
                            base.object = PathBuf::from(&object_path);
                            let operator = Operator::from_map(base.backend.clone()).build().unwrap();

                            exporters.push(S3Core {
                                op: Some(operator),
                                base,
                            });
                        },
                    }
                }

                exporters
            }
            Self::BitWarden => {
                let mut exporters = vec![];

                while continue_loop(&exporters, "BitWarden account") {
                    // TODO :: Possibly users but definitely organizations
                }

                exporters
            }
        };

        Ok(exporters
            .into_iter()
            .map(|exporter| Box::new(exporter) as Box<dyn Backend>)
            .collect())
    }
}

pub trait Backend: Serialize + Deserialize + Downloader + Prune + Display + Debug {}

#[async_trait]
pub trait Downloader
where
    Self: Display + Debug,
{
    /// This method will download the backup data into memory,
    /// and then write it to the backup directory.
    async fn download(
        &self,
        root_directory: &PathBuf,
        auto_prune: &AutoPrune,
    ) -> anyhow::Result<()>;
}
