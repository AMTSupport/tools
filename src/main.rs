use anyhow::{Context, Result};
use cleaner::log::init_loggers;
use dirs::template_dir;
use simplelog::info;
use std::borrow::ToOwned;
use std::env::temp_dir;
use std::fs::File;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // TODO :: Command Parser

    // TODO :: Config Parser

    init_loggers()?;

    info!("Starting cleaner");

    Ok(())
}
