use crate::sources::{env_or_prompt, Exporter};
use lib::anyhow::{Context, Result};
use opendal::layers::LoggingLayer;
use opendal::{Builder, Operator, OperatorBuilder};
use std::path::PathBuf;

pub struct S3Exporter {
    op: Operator,
}

#[derive(Debug, Clone)]
pub struct SubS3Exporter {
    op: Operator,
    to: PathBuf,
    object: PathBuf,
}

impl S3Exporter {
    pub fn new(interactive: &bool) -> Result<Self> {
        let bucket = env_or_prompt("S3_BUCKET", "S3 Bucket Name", false, &interactive, |_| true)?;
        let region = env_or_prompt("S3_REGION", "S3 Region", false, &interactive, |_| true)?;
        let endpoint = env_or_prompt("S3_ENDPOINT", "S3 Endpoint", false, &interactive, |_| true)?;
        let access_key_id = env_or_prompt(
            "S3_ACCESS_KEY_ID",
            "S3 Access Key ID",
            false,
            &interactive,
            |_| true,
        )?;
        let secret_access_key = env_or_prompt(
            "S3_SECRET_ACCESS_KEY",
            "S3 Access Key",
            true,
            &interactive,
            |_| true,
        )?;

        let backend = opendal::services::S3::default()
            .bucket(&bucket)
            .region(&region)
            .endpoint(&endpoint)
            .access_key_id(&access_key_id)
            .secret_access_key(&secret_access_key)
            .build()
            .context("Failed to create S3 Backend")?;

        let op = OperatorBuilder::new(backend)
            .layer(LoggingLayer::default())
            .finish();

        Ok(Self { op })
    }

    pub fn for_object(&self, backup_root: &PathBuf, objects_path: PathBuf) -> SubS3Exporter
    where
        SubS3Exporter: Exporter,
    {
        // TODO :: Bucket name
        let to_path = backup_root.join("S3").join(&objects_path);
        let object_path = objects_path.clone();

        SubS3Exporter {
            op: self.op.clone(),
            to: to_path,
            object: object_path,
        }
    }
}

impl Exporter for SubS3Exporter {
    fn prune(&self) -> Result<()> {
        todo!()
    }

    fn download(&self) -> Result<()> {
        todo!()
    }
}
