use lib::anyhow;
use lib::anyhow::anyhow;
use lib::simplelog::info;
use std::fmt::{Debug, Display, Formatter};
use std::path::PathBuf;
use clap::ValueEnum;
use opendal::Operator;
use serde::{Deserialize, Serialize};

pub mod s3;

#[derive(Debug, Clone)]
pub enum ExporterSource {
    S3 { op: Operator, to: PathBuf, object: PathBuf },
}

impl Display for ExporterSource {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::S3 => write!(f, "S3"),
        }
    }
}

impl ExporterSource {
    pub fn create(
        &self,
        interactive: &bool,
        root_path: &PathBuf,
    ) -> anyhow::Result<Vec<impl Exporter + Debug>> {
        match self {
            Self::S3 => {
                let exporter = s3::S3Exporter::new(&interactive)?;
                let mut sub_exporters = vec![];

                loop {
                    if !sub_exporters.is_empty() {
                        let more =
                            inquire::Confirm::new("Do you want to add another object to export?")
                                .with_default(false)
                                .prompt()?;

                        if more == false {
                            break;
                        }
                    }

                    match inquire::Text::new("What's the path of the object you want to export?")
                        .prompt()?
                    {
                        object_path if object_path.is_empty() => {
                            info!("Assuming wanted to cancel additional object.");
                            continue;
                        }
                        object_path => sub_exporters
                            .push(exporter.for_object(root_path, PathBuf::from(object_path))),
                    }
                }

                Ok(sub_exporters)
            }
        }
    }
}

pub trait Exporter {
    /// This will search the backup directory for existing backup data.
    /// If there is backup data which is beyond the retention period, it will be deleted.
    fn prune(&self) -> anyhow::Result<()>;

    /// This method will download the backup data into memory,
    /// and then write it to the backup directory.
    fn download(&self) -> anyhow::Result<()>;
}

fn env_or_prompt(
    key: &str,
    prompt_title: &str,
    sensitive_info: bool,
    interactive: &bool,
    validator: fn(&String) -> bool,
) -> anyhow::Result<String> {
    let value = std::env::var(key);
    if let Ok(value) = value {
        if !validator(&value) {
            return Err(anyhow!("{} is not valid", key))?;
        }

        return Ok(value);
    }

    if !interactive {
        return Err(anyhow!(
            "{} is not set and interactive mode is disabled",
            key
        ))?;
    }

    let prompt = match sensitive_info {
        true => inquire::Password::new(prompt_title).prompt(),
        false => inquire::Text::new(prompt_title).prompt(),
    };

    match prompt {
        Ok(value) => Ok(value),
        Err(err) => Err(anyhow!("Failed to get {} from user: {}", key, err))?,
    }
}
