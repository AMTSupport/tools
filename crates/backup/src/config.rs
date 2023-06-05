use std::any::{Any, TypeId};
use crate::sources::{Exporter, ExporterSource};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use crate::sources::s3::SubS3Exporter;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    exporters: Vec<ExporterConfig>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExporterConfig {
    source_config: Value,
}

impl ExporterConfig {
    fn serialise(exporter: impl Exporter) -> Self {
        match exporter {
            SubS3Exporter { op, to, object } => {
                let source_config = serde_json::json!({
                    "to": to,
                    "object": object,
                });

                Self {
                    source_type: ExporterSource::S3,
                    source_config,
                }
            }
        }

        Self {

        }
    }

    fn deserialise<T>(&self) -> T where T: Exporter + Sized + 'static {


        T::new()
    }
}
