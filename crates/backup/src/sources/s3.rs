/*
 * Copyright (C) 2023 James Draycott <me@racci.dev>
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, version 3.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */

use crate::config::backend::Backend;
use crate::config::runtime::RuntimeConfig;
use crate::sources::auto_prune::Prune;
use crate::sources::download_to;
use crate::sources::exporter::Exporter;
use crate::{continue_loop, env_or_prompt};
use async_trait::async_trait;
use chrono::Utc;
use futures::{Stream, TryStreamExt};
use futures_util::StreamExt;
use indicatif::{MultiProgress, ProgressBar};
use inquire::validator::Validation;
use lib::anyhow::{Context, Result};
use lib::fs::normalise_path;
use lib::progress::{download, spinner};
use opendal::layers::LoggingLayer;
use opendal::services::S3;
use opendal::{Builder, Operator, OperatorBuilder};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::string::ToString;
use std::time::UNIX_EPOCH;
use tracing::{debug, error, info, trace};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct S3Base {
    pub object: PathBuf,

    /// Keeps a serializable map of the operators data,
    /// since we can't serialize the operator itself.
    #[serde(flatten)]
    pub backend: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct S3Core {
    #[serde(skip)]
    pub(crate) op: Option<Operator>,
    #[serde(flatten)]
    pub base: S3Base,
}

impl PartialEq for S3Core {
    fn eq(&self, other: &Self) -> bool {
        self.base == other.base
    }
}

impl Eq for S3Core {}

impl S3Core {
    fn op(&mut self) -> &Operator {
        self.op.get_or_insert_with(|| {
            let backend = S3::from_map(self.base.backend.clone()).build().unwrap();
            OperatorBuilder::new(backend).layer(LoggingLayer::default()).finish()
        })
    }
}

#[async_trait]
impl Prune for S3Core {
    fn files(&self, config: &RuntimeConfig) -> Result<Vec<PathBuf>> {
        use std::path::MAIN_SEPARATOR;

        let glob = format!(
            "{root}{MAIN_SEPARATOR}{bucket}{MAIN_SEPARATOR}*",
            root = normalise_path(S3Core::base_dir(config)).display(),
            bucket = self.base.object.display()
        );

        glob::glob(&glob)
            .with_context(|| format!("Failed to glob: {}", glob))
            .map(|p| p.flatten().collect())
    }
}

#[async_trait]
impl Exporter for S3Core {
    const DIRECTORY: &'static str = "S3";

    async fn interactive(_config: &RuntimeConfig) -> Result<Vec<Backend>> {
        let not_empty_or_ascii = |str: &str, msg: &str| match str
            .chars()
            .any(|c| !c.is_ascii_alphanumeric() && c != '-' && c != '_')
            || str.is_empty()
        {
            false => Ok(Validation::Valid),
            true => Ok(Validation::Invalid(msg.into())),
        };

        let bucket = env_or_prompt("S3_BUCKET", move |str: &_| {
            not_empty_or_ascii(
                str,
                "Bucket name must be alphanumeric, and can only contain dashes and underscores.",
            )
        })?;

        // TODO Validators
        let region = env_or_prompt("S3_REGION", |_: &_| Ok(Validation::Valid))?;
        let endpoint = env_or_prompt("S3_ENDPOINT", |_: &_| Ok(Validation::Valid))?;
        let key_id = env_or_prompt("S3_ACCESS_KEY_ID", |_: &_| Ok(Validation::Valid))?;
        let secret_key = env_or_prompt("S3_SECRET_ACCESS_KEY", |_: &_| Ok(Validation::Valid))?;

        let base_accessor = HashMap::from([
            ("bucket".to_string(), bucket),
            ("region".to_string(), region),
            ("endpoint".to_string(), endpoint),
            ("access_key_id".to_string(), key_id),
            ("secret_access_key".to_string(), secret_key), // TODO :: This is not secure at all, maybe use platform specific keychain?
        ]);

        let base = S3Base {
            object: PathBuf::from(""),
            backend: base_accessor,
        };

        let prompt = inquire::Text::new("What's the path of the object you want to export?")
            .with_validator(|path: &str| match path.is_empty() || !path.ends_with('/') {
                true => Ok(Validation::Invalid("Path must end with /".into())),
                false => Ok(Validation::Valid),
            });

        // TODO :: Auto suggest for object paths
        let mut exporters = vec![];
        while continue_loop(&exporters, "object to export") {
            match prompt.clone().prompt()? {
                object_path if object_path.is_empty() => {
                    info!("Assuming wanted to cancel additional object.");
                    continue;
                }
                object_path => {
                    let mut base = base.clone();
                    base.backend.insert("root".to_string(), object_path.clone());
                    base.object = PathBuf::from(&object_path);

                    let operator = match Operator::from_map::<S3>(base.backend.clone()) {
                        Ok(b) => b.layer(LoggingLayer::default()).finish(),
                        Err(e) => {
                            error!("Failed to interactive operator: {}", e);
                            continue;
                        }
                    };

                    exporters.push(Backend::S3(S3Core {
                        op: Some(operator),
                        base,
                    }));
                }
            }
        }

        Ok(exporters)
    }

    // TODO :: Validate files
    async fn export(
        &mut self,
        config: &RuntimeConfig,
        main_bar: &ProgressBar,
        progress_bar: &MultiProgress,
    ) -> Result<()> {
        let progress_state = progress_bar.insert_after(main_bar, spinner());
        progress_state.set_message("Initialising S3 exporter...");

        let object = self.base.object.clone();
        let output = normalise_path(Self::base_dir(config).join(&object));
        let mut backup_len = self.files(config)?.len();
        let op = self.op();

        progress_state.set_message("Requesting objects from S3...");

        // TODO :: Should this be recursive?
        let mut layer = op.list_with("/").await.context(format!(
            "Failed to list objects in {}",
            &object.to_str().unwrap()
        ))?;

        progress_state.set_message("Processing objects from S3...");
        progress_state.set_length(layer.size_hint().1.unwrap_or(0) as u64);
        progress_state.set_position(0);
        let download_bar = progress_bar.insert_after(&progress_state, download());

        while let Some(item) = layer.try_next().await? {
            let meta = op.metadata(&item, None).await?;
            if meta.is_dir() {
                progress_state.inc(1);
                continue;
            }

            let path = normalise_path(output.join(item.name()));
            let filename = path.file_name().unwrap().to_str().unwrap();
            progress_state.set_message(format!("Processing {:#}", &filename));

            if path.exists() {
                debug!("Checking if file has changed...");
                progress_state.set_message(format!("Checking if {:#} has changed...", &filename));

                let host_meta = std::fs::metadata(&path)?;
                let host_len = host_meta.len();
                let host_modified = host_meta.modified().unwrap().duration_since(UNIX_EPOCH)?;
                let remote_len = meta.content_length();
                let remote_modified = meta.last_modified().unwrap();

                debug!("Host len: {}", host_len);
                debug!("Host modified: {}", host_modified.as_millis());
                debug!("Remote len: {}", remote_len);
                debug!(
                    "Remote modified: {}",
                    remote_modified.timestamp_millis() as u128
                );

                if host_len == remote_len
                    && host_modified.as_millis() == remote_modified.timestamp_millis() as u128
                {
                    debug!("Skipping export as file is the same");
                    progress_state.inc(1);
                    continue;
                }

                if host_len != remote_len {
                    debug!("File size is different, deleting");
                    std::fs::remove_file(&path)?;
                    backup_len -= 1;
                }

                progress_state.inc(1)
            }

            debug!("Checking if file should be pruned...");
            progress_state.set_message(format!("Checking if {:#} would be pruned...", &filename));

            if config.config.rules.auto_prune.enabled
                && backup_len > config.config.rules.auto_prune.keep_latest
            {
                let since_mtime = Utc::now() - meta.last_modified().unwrap();
                if since_mtime.num_days() > config.config.rules.auto_prune.days as i64 {
                    debug!(
                        "File is older than {}, skipping.",
                        &config.config.rules.auto_prune.days
                    );
                    progress_state.inc(1);
                    continue;
                }
            }

            trace!("Checking if {} exists", &path.to_str().unwrap());
            let host_path = normalise_path(config.directory.join(&path));
            if host_path.exists() && meta.content_length() == host_path.metadata().unwrap().len() {
                debug!(
                    "Skipping export of {} as it already exists",
                    &path.to_str().unwrap()
                );
                progress_state.inc(1);
                continue;
            }

            progress_state.set_message(format!("Downloading {:#}...", &filename));
            let reader = op.reader_with(item.path()).await?;
            download_to(meta.content_length(), reader.boxed(), &path, &download_bar).await?;

            debug!("Setting access time for {}", &path.to_str().unwrap());
            progress_state.set_message(format!("Setting access time for {:#}...", &filename));

            let access_time = meta.last_modified().unwrap();
            filetime::set_file_mtime(
                &path,
                filetime::FileTime::from_system_time(access_time.into()),
            )
            .with_context(|| {
                format!("Failed to set access time for {}", &path.to_str().unwrap())
            })?;

            progress_state.inc(1);
            backup_len += 1;
        }

        download_bar.finish_and_clear();

        Ok(())
    }
}
