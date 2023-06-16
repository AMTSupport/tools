use bytes::Bytes;
use futures_util::stream::BoxStream;
use futures_util::StreamExt;
use indicatif::{ProgressBar, ProgressStyle};
use lib::anyhow::{anyhow, Context};
use lib::progress::download_style;
use lib::{anyhow, progress};
use rand::RngCore;
use std::cmp::min;
use std::error::Error;
use std::io::Write;
use std::path::PathBuf;
use std::{env, fs};

pub mod auto_prune;
pub mod bitwarden;
pub mod exporter;
pub(crate) mod interactive;
pub mod op;
pub mod s3;

async fn download_to<E: Error>(
    total_size: u64,
    mut stream: BoxStream<'_, Result<Bytes, E>>,
    path: &PathBuf,
    progress: &ProgressBar,
) -> anyhow::Result<()> {
    progress.set_length(total_size);
    let mut file = fs::File::create(&path)?;
    let mut downloaded = 0u64;

    progress.set_message(format!(
        "Downloading {}...",
        &path.file_name().unwrap().to_str().unwrap()
    ));

    while let Some(item) = stream.next().await {
        let chunk = item.or(Err(anyhow!("Error downloading file")))?;
        file.write_all(&chunk).context("Error writing to temporary file")?;
        let new = min(downloaded + (chunk.len() as u64), total_size);
        downloaded = new;
        progress.set_position(new);
    }

    progress.set_message("Download complete");

    Ok(())
}

async fn download<E: Error>(
    total_size: u64,
    mut stream: BoxStream<'_, Result<Bytes, E>>,
    progress: &ProgressBar,
) -> anyhow::Result<PathBuf> {
    let path = env::temp_dir().join(format!("download-{}", rand::thread_rng().next_u64()));
    download_to(total_size, stream, &path, progress).await?;
    Ok(path)
}
