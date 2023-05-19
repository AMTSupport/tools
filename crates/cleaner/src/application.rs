use crate::builder::CleanableBuilderTrait;
use crate::{Indexed, LOCATIONS};
use lib::cli::Flags;
use log::info;
use tokio_stream::{self as stream, StreamExt};

// TODO :: Maybe change for windows since it reports sizes differently.
const KILOBYTE: f64 = 1024f64;
const MEGABYTE: f64 = 1024f64 * KILOBYTE;
const GIGABYTE: f64 = 1024f64 * MEGABYTE;

pub async fn application(cli: Flags) -> anyhow::Result<()> {
    info!("Starting cleaner");

    let mut cleanable = Vec::with_capacity(LOCATIONS.len());
    let mut stream = stream::iter(LOCATIONS.iter());
    while let Some(builder) = stream.next().await {
        let built = builder.clone().build();
        match built {
            Ok(cleaner) => {
                cleanable.push(cleaner);
                info!("Built cleaner for {:?}", builder.composing);
            }
            Err(e) => info!("Failed to build cleaner: {}", e),
        }
    }

    let mut auto_size = 0f64;
    let mut auto_files = 0usize;
    let mut manual_size = 0f64;
    let mut manual_files = 0usize;
    for cleaner in cleanable {
        let (inner_auto_files, inner_auto_size, inner_manual_files, inner_manual_size) =
            cleaner.clean(&cli).await;

        auto_files += inner_auto_files;
        auto_size += inner_auto_size;
        manual_files += inner_manual_files;
        manual_size += inner_manual_size;
    }

    let size_str = |size| {
        if size >= GIGABYTE {
            format!("{:.2} GB", size / GIGABYTE)
        } else if size >= MEGABYTE {
            format!("{:.2} MB", size / MEGABYTE)
        } else if size >= KILOBYTE {
            format!("{:.2} KB", size / KILOBYTE)
        } else {
            format!("{:.2} B", size)
        }
    };

    info!(
        "Cleaned up {} files, freeing up {}",
        auto_files,
        size_str(auto_size)
    );
    info!(
        "Additional {} files, able to clean up {}",
        manual_files,
        size_str(manual_size)
    );

    Ok(())
}
