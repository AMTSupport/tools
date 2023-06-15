use crate::config::runtime::RuntimeConfig;
use async_trait::async_trait;
use lib::anyhow::Result;

#[async_trait]
pub trait Interactive<T> {
    /// Creates a new async function which will prompt the user for the required information to create the exporter;
    async fn interactive(config: &RuntimeConfig) -> Result<T>;
}
