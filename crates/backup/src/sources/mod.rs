use bytes::Bytes;
use futures_util::stream::BoxStream;
use futures_util::StreamExt;
use indicatif::ProgressStyle;
use lib::anyhow;
use lib::anyhow::{anyhow, Context};
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
) -> anyhow::Result<()> {
    let pb = indicatif::ProgressBar::new(total_size)
        .with_style(ProgressStyle::default_bar()
            .progress_chars("#>-")
            .template("{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec}, {eta})")?);

    let mut file = fs::File::create(&path)?;
    let mut downloaded = 0u64;

    while let Some(item) = stream.next().await {
        let chunk = item.or(Err(anyhow!("Error downloading file")))?;
        file.write_all(&chunk)
            .context("Error writing to temporary file")?;
        let new = min(downloaded + (chunk.len() as u64), total_size);
        downloaded = new;
        pb.set_position(new);
    }

    pb.finish_and_clear();
    Ok(())
}

async fn download<E: Error>(
    total_size: u64,
    mut stream: BoxStream<'_, Result<Bytes, E>>,
) -> anyhow::Result<PathBuf> {
    let path = env::temp_dir().join(format!("download-{}", rand::thread_rng().next_u64()));
    download_to(total_size, stream, &path).await?;
    Ok(path)
}
