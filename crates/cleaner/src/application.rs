use crate::builder::CleanableBuilderTrait;
use crate::{clean, Indexed, PreparedPaths, LOCATIONS};
use clap::Parser;
use indicatif::ParallelProgressIterator;
use lib::cli::Flags;
use rayon::prelude::*;
use simplelog::info;
use std::sync::{Arc, Mutex};

#[derive(Parser, Debug)]
#[command(name = env!["CARGO_PKG_NAME"], version, author, about)]
pub struct Cli {
    // Allows a user to interact with the application.
    #[arg(short, long)]
    pub interactive: bool,

    #[command(flatten)]
    pub flags: Flags,
}

pub async fn application(cli: Cli) -> anyhow::Result<()> {
    info!("Starting cleaner");

    // TODO :: Progress bar
    info!("Collecting cleanable items...");
    let cleanable = LOCATIONS
        .clone()
        .into_par_iter()
        .progress_count(LOCATIONS.len() as u64)
        .with_message("Collecting cleanable items...")
        .with_style(
            indicatif::ProgressStyle::default_spinner()
                .progress_chars("#>-")
                .template("{spinner:.green} [{bar:40.cyan/blue}] {pos:>7}/{len:7} {msg}")?,
        )
        .filter_map(|builder| builder.build().ok())
        .collect::<Vec<_>>();

    info!("Preparing for clean-up...");
    let auto = Arc::new(Mutex::new(PreparedPaths::default()));
    let manual = Arc::new(Mutex::new(PreparedPaths::default()));

    let multi_progress = indicatif::MultiProgress::new();
    let spinner = || {
        indicatif::ProgressBar::new_spinner().with_style(
            indicatif::ProgressStyle::default_spinner()
                .progress_chars("#>-")
                .template("{spinner:.green} [{bar:40.cyan/blue}] {pos:>7}/{len:7} {msg}")
                .unwrap(),
        )
    };

    fn proc(prep: Arc<Mutex<PreparedPaths>>, inner_prep: PreparedPaths) {
        let mut prep = prep.lock().unwrap();
        prep.merge_with(inner_prep);
    }

    cleanable
        .par_iter()
        .map(|cleanable| {
            let spinner = multi_progress.add(spinner());
            cleanable.prepare(&cli.flags, spinner).unwrap()
        })
        .collect::<Vec<_>>()
        .into_iter()
        .for_each(|(inner_auto, inner_manual)| {
            proc(auto.clone(), inner_auto);
            proc(manual.clone(), inner_manual);
        });

    let mut auto = auto.lock().unwrap();
    let mut manual = manual.lock().unwrap();
    let mut missed_size = 0u64;

    if !cli.flags.dry_run {
        info!("Cleaning up...");
        let uncleaned_size = clean(&auto)?;
        auto.disk_size -= uncleaned_size;
        missed_size += uncleaned_size;
    } else {
        info!("Dry run, no files have been <red>deleted</> or <yellow>modified</>.");
    }

    info!(
        "Automatic cleanup removed {removed_files} files, freeing up a total of {removed_size}.",
        removed_files = indicatif::HumanCount(auto.paths.len() as u64),
        removed_size = indicatif::HumanBytes(auto.disk_size)
    );

    if missed_size > 0 {
        info!(
            "Automatic cleanup was unable to remove some files, which would have freed up an additional total of {missed_size}.",
            missed_size = indicatif::HumanBytes(missed_size)
        )
    }

    if !manual.paths.is_empty() {
        info!(
            "There are <blue>{additional_files}</> files which require manual cleanup approval,\
            These files would clean up a total of <blue>{additional_size}</> if <red>removed</>.",
            additional_files = indicatif::HumanCount(manual.paths.len() as u64),
            additional_size = indicatif::HumanBytes(manual.disk_size),
        );
    }

    // Only prompt for manual marked files if we are in interactive mode.
    if !cli.interactive || manual.paths.is_empty() {
        return Ok(());
    }

    match inquire::Confirm::new("Do you want to clean up the additional files?")
        .with_default(false)
        .prompt()
    {
        Ok(true) => {
            // Display the additional files and prompt for confirmation.
            // Maybe allow selecting which files to clean up?
            let fmt_paths = manual.paths.iter().map(|path| path.display()).collect::<Vec<_>>();
            let select =
                inquire::MultiSelect::new("Select additional files to clean up.", fmt_paths)
                    .with_page_size(15);
            match select.prompt() {
                Ok(selection) => {
                    let selection = selection
                        .iter()
                        .map(|display| {
                            manual
                                .paths
                                .iter()
                                .find(|item| item.display().to_string() == display.to_string())
                                .unwrap()
                                .clone()
                        })
                        .collect::<Vec<_>>();
                    manual.paths = selection;

                    info!("Cleaning up additional files...");
                    let uncleaned_size = clean(&manual)?;
                    manual.disk_size -= uncleaned_size;

                    info!(
                        "Managed to free up an additional {}.",
                        indicatif::HumanBytes(manual.disk_size)
                    );
                }
                Err(e) => info!("Failed to prompt for additional files: {}", e),
            }
        }
        Ok(false) => return Ok(()),
        Err(e) => info!("Failed to prompt for additional files: {}", e),
    }

    Ok(())
}
