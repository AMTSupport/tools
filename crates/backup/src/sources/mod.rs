use bytes::Bytes;
use futures_util::stream::BoxStream;
use futures_util::StreamExt;
use indicatif::ProgressBar;
use lib::anyhow;
use lib::anyhow::{anyhow, Context};
use lib::fs::{create_parents, normalise_path};
use lib::simplelog::debug;
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
pub mod downloader;

async fn download_to<E: Error>(
    total_size: u64,
    mut stream: BoxStream<'_, Result<Bytes, E>>,
    path: &PathBuf,
    progress: &ProgressBar,
) -> anyhow::Result<()> {
    debug!("Creating parent dir for {}", &path.display());
    create_parents(path)?;

    progress.set_length(total_size);
    let mut downloaded = 0u64;
    let mut file =
        fs::File::create(path).with_context(|| format!("Create file {}", path.display()))?;

    progress.set_message(format!(
        "Downloading {}...",
        &path.file_name().context("Get file name")?.to_str().context("Convert to string")?
    ));

    while let Some(item) = stream.next().await {
        let chunk = item.or(Err(anyhow!("Error downloading file")))?;
        file.write_all(&chunk).context("Error writing to file")?;
        let new = min(downloaded + (chunk.len() as u64), total_size);
        downloaded = new;
        progress.set_position(new);
    }

    progress.set_message("Download complete");

    Ok(())
}

async fn download<E: Error>(
    total_size: u64,
    stream: BoxStream<'_, Result<Bytes, E>>,
    progress: &ProgressBar,
) -> anyhow::Result<PathBuf> {
    let path = env::temp_dir().join(format!("download-{}", rand::thread_rng().next_u64()));
    let path = normalise_path(path);
    download_to(total_size, stream, &path, progress).await?;
    Ok(path)
}
