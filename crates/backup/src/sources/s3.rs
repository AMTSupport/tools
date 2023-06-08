use crate::config::AutoPrune;
use crate::sources::auto_prune::Prune;
use crate::sources::{Backend, Downloader, ExporterSource};
use crate::{env_or_prompt, trackable_filename};
use async_trait::async_trait;
use chrono::{DateTime, FixedOffset, Utc};
use futures::{StreamExt, TryStreamExt};
use lib::anyhow::{Context, Result};
use lib::simplelog::{debug, trace};
use opendal::layers::LoggingLayer;
use opendal::services::S3;
use opendal::{Builder, Operator, OperatorBuilder};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::HashMap;
use std::fmt::Display;
use std::io::Write;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct S3Base {
    pub object: PathBuf,

    /// Keeps a serializable map of the operators data,
    /// since we can't serialize the operator itself.
    #[serde(flatten)]
    pub backend: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct S3Core {
    #[serde(skip)]
    pub(crate) op: Option<Operator>,
    #[serde(flatten)]
    pub base: S3Base,
}

impl S3Core {
    fn op(&mut self) -> &Operator {
        self.op.get_or_insert_with(|| {
            let backend = S3::from_map(self.base.backend.clone()).build().unwrap();
            OperatorBuilder::new(backend)
                .layer(LoggingLayer::default())
                .finish()
        })
    }
}

impl Backend for S3Core {}

impl Display for S3Core {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}: {}",
            ExporterSource::S3,
            self.object.to_str().unwrap()
        )
    }
}

impl Prune for S3Core {
    fn files(&self, root_directory: &PathBuf) -> Vec<PathBuf> {
        let directory = root_directory.join("S3").join(&self.object);
        if !directory.exists() {
            return vec![];
        }

        std::fs::read_dir(directory)
            .unwrap()
            .filter_map(|entry| entry.ok().map(|entry| entry.path()))
            .collect()
    }
}

impl Serialize for S3Core {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut accessor = self.operator.accessor.clone();
        accessor.insert(
            "object".to_string(),
            self.object.to_str().unwrap().to_string(),
        );

        accessor.serialize(serializer)
    }
}

#[async_trait]
impl Downloader for S3Core {
    async fn download(&self, root_directory: &PathBuf, auto_prune: &AutoPrune) -> Result<()> {
        let op = self.operator.op.as_ref().unwrap();
        let object = self.object.clone();
        let output = root_directory.join("S3");
        let mut backup_len = self.files(&root_directory).len();

        // TODO :: Should this be recursive?
        let mut layer = op
            .list_with(object.to_str().unwrap())
            .await
            .context(format!(
                "Failed to list objects in {}",
                &object.to_str().unwrap()
            ))?;

        while let Some(item) = layer.try_next().await? {
            debug!("Processing {:?}", &item);

            let meta = op.metadata(&item, None).await?;
            if meta.is_dir() {
                continue;
            }

            let path = output.join(&item.path());
            debug!("Working with item at {:?}", &path);

            if path.exists() {
                debug!("Item at path exists");
                let host_meta = std::fs::metadata(&path)?;
                let host_len = host_meta.len();
                let host_modified = host_meta.modified().unwrap().duration_since(UNIX_EPOCH)?;
                let remote_len = meta.content_length();
                let remote_modified = meta.last_modified().unwrap();

                if host_len == remote_len
                    && host_modified.as_millis() == remote_modified.timestamp_millis() as u128
                {
                    debug!("Skipping download as file is the same");
                    continue;
                }

                if host_len != remote_len {
                    debug!("File size is different, deleting");
                    std::fs::remove_file(&path)?;
                    backup_len -= 1;
                }
            }

            if auto_prune.enabled.clone() && &backup_len > &auto_prune.keep_latest {
                let since_mtime = Utc::now() - meta.last_modified().unwrap();
                if since_mtime.num_days() > auto_prune.keep_for.clone() as i64 {
                    debug!("File is older than {}, skipping.", &auto_prune.keep_for);
                    continue;
                }
            }

            trace!("Checking if {} exists", &path.to_str().unwrap());
            let host_path = root_directory.join(&path);
            if host_path.exists() && meta.content_length() == host_path.metadata().unwrap().len() {
                debug!(
                    "Skipping download of {} as it already exists",
                    &path.to_str().unwrap()
                );
                continue;
            }

            debug!("Creating parent dir for {}", &path.to_str().unwrap());
            std::fs::create_dir_all(&path.parent().unwrap())
                .context("Unable to create directory")?;

            debug!("Creating file {}", &path.to_str().unwrap());
            let mut file = std::fs::File::create(&path)?;
            let mut reader = op.reader_with(&item.path()).await?;
            while let Some(chunk) = reader.try_next().await? {
                file.write_all(&chunk)?;
            }

            debug!("Setting access time for {}", &path.to_str().unwrap());
            let access_time = meta.last_modified().unwrap();
            filetime::set_file_mtime(
                &path,
                filetime::FileTime::from_system_time(access_time.into()),
            )
            .with_context(|| {
                format!("Failed to set access time for {}", &path.to_str().unwrap())
            })?;

            backup_len += 1;
        }

        Ok(())
    }
}

pub fn create_operator(interactive: &bool) -> Result<S3Core> {
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
        false,
        &interactive,
        |_| true,
    )?;

    let backend = S3::default()
        .bucket(&bucket)
        .region(&region)
        .endpoint(&endpoint)
        .access_key_id(&access_key_id)
        .secret_access_key(&secret_access_key)
        .build()
        .context("Failed to create S3 Backend")?;

    let accessor = HashMap::from([
        ("bucket".to_string(), bucket),
        ("region".to_string(), region),
        ("endpoint".to_string(), endpoint),
        ("access_key_id".to_string(), access_key_id),
        ("secret_access_key".to_string(), secret_access_key), // TODO :: This is not secure at all, maybe use platform specific keychain?
    ]);

    let op = OperatorBuilder::new(backend)
        .layer(LoggingLayer::default())
        .layer(opendal::layers::RetryLayer::default())
        .finish();

    Ok(S3Core {
        op: None,
        base: S3Base {
            object: PathBuf::new(),
            backend: accessor,
        },
    })
}
