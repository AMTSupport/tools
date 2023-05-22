use chrono::{DateTime, Utc};
use serde::{Deserialize, Deserializer};

pub mod hudu;
pub mod nable;

const AGENT: &str = "rest_agent";

#[derive(Debug)]
pub struct Client {
    base_url: String,
    api_key: String,
    client: reqwest::Client,
}

pub fn deserialise_datetime<'de, D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    chrono::DateTime::parse_from_rfc3339(&s)
        .map(|dt| dt.with_timezone(&Utc))
        .map_err(serde::de::Error::custom)
}

pub trait Url<C: ?Sized> {
    fn link(&self, client: &C) -> String;
}
