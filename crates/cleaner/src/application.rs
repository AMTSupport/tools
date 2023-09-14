/*
 * Copyright (C) 2023 James Draycott <me@racci.dev>
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, version 3.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */

use crate::cleaners::cleaner::{CleanupResult, SkipReason};
use crate::config::runtime::Runtime;
use tokio_stream::StreamExt;
use tracing::{debug, error, info};

pub async fn application(runtime: &'static Runtime) -> anyhow::Result<()> {
    info!("Starting cleaner");

    let results = run_cleaners(runtime).await;
    write_result(results);

    // let cleaners =
    //
    // let cleanable = LOCATIONS
    //     .clone()
    //     .into_par_iter()
    //     .progress_count(LOCATIONS.len() as u64)
    //     .with_message("Collecting cleanable items...")
    //     .with_style(
    //         indicatif::ProgressStyle::default_spinner()
    //             .progress_chars("#>-")
    //             .template("{spinner:.green} [{bar:40.cyan/blue}] {pos:>7}/{len:7} {msg}")?,
    //     )
    //     .filter_map(|builder| builder.build().ok())
    //     .collect::<Vec<_>>();
    //
    // info!("Preparing for clean-up...");
    // let auto = Arc::new(Mutex::new(PreparedPaths::default()));
    // let manual = Arc::new(Mutex::new(PreparedPaths::default()));
    //
    // let multi_progress = indicatif::MultiProgress::new();
    // let spinner = || {
    //     indicatif::ProgressBar::new_spinner().with_style(
    //         indicatif::ProgressStyle::default_spinner()
    //             .progress_chars("#>-")
    //             .template("{spinner:.green} [{bar:40.cyan/blue}] {pos:>7}/{len:7} {msg}")
    //             .unwrap(),
    //     )
    // };
    //
    // fn proc(prep: Arc<Mutex<PreparedPaths>>, inner_prep: PreparedPaths) {
    //     let mut prep = prep.lock().unwrap();
    //     prep.merge_with(inner_prep);
    // }
    //
    // cleanable
    //     .par_iter()
    //     .map(|cleanable| {
    //         let spinner = multi_progress.add(spinner());
    //         cleanable.prepare(&cli.flags, spinner).unwrap()
    //     })
    //     .collect::<Vec<_>>()
    //     .into_iter()
    //     .for_each(|(inner_auto, inner_manual)| {
    //         proc(auto.clone(), inner_auto);
    //         proc(manual.clone(), inner_manual);
    //     });
    //
    // let mut auto = auto.lock().unwrap();
    // let mut manual = manual.lock().unwrap();
    // let mut missed_size = 0u64;
    //
    // if !cli.flags.dry_run {
    //     info!("Cleaning up...");
    //     let uncleaned_size = clean(&auto)?;
    //     auto.disk_size -= uncleaned_size;
    //     missed_size += uncleaned_size;
    // } else {
    //     info!("Dry run, no files have been <red>deleted</> or <yellow>modified</>.");
    // }
    //
    // info!(
    //     "Automatic cleanup removed {removed_files} files, freeing up a total of {removed_size}.",
    //     removed_files = indicatif::HumanCount(auto.paths.len() as u64),
    //     removed_size = indicatif::HumanBytes(auto.disk_size)
    // );
    //
    // if missed_size > 0 {
    //     info!(
    //         "Automatic cleanup was unable to remove some files, which would have freed up an additional total of {missed_size}.",
    //         missed_size = indicatif::HumanBytes(missed_size)
    //     )
    // }
    //
    // if !manual.paths.is_empty() {
    //     info!(
    //         "There are <blue>{additional_files}</> files which require manual cleanup approval,\
    //         These files would clean up a total of <blue>{additional_size}</> if <red>removed</>.",
    //         additional_files = indicatif::HumanCount(manual.paths.len() as u64),
    //         additional_size = indicatif::HumanBytes(manual.disk_size),
    //     );
    // }
    //
    // // Only prompt for manual marked files if we are in interactive mode.
    // if !cli.interactive || manual.paths.is_empty() {
    //     return Ok(());
    // }
    //
    // match inquire::Confirm::new("Do you want to clean up the additional files?")
    //     .with_default(false)
    //     .prompt()
    // {
    //     Ok(true) => {
    //         // Display the additional files and prompt for confirmation.
    //         // Maybe allow selecting which files to clean up?
    //         let fmt_paths = manual.paths.iter().map(|paths| paths.display()).collect::<Vec<_>>();
    //         let select =
    //             inquire::MultiSelect::new("Select additional files to clean up.", fmt_paths)
    //                 .with_page_size(15);
    //         match select.prompt() {
    //             Ok(selection) => {
    //                 let selection = selection
    //                     .iter()
    //                     .map(|display| {
    //                         manual
    //                             .paths
    //                             .iter()
    //                             .find(|item| item.display().to_string() == display.to_string())
    //                             .unwrap()
    //                             .clone()
    //                     })
    //                     .collect::<Vec<_>>();
    //                 manual.paths = selection;
    //
    //                 info!("Cleaning up additional files...");
    //                 let uncleaned_size = clean(&manual)?;
    //                 manual.disk_size -= uncleaned_size;
    //
    //                 info!(
    //                     "Managed to free up an additional {}.",
    //                     indicatif::HumanBytes(manual.disk_size)
    //                 );
    //             }
    //             Err(e) => info!("Failed to prompt for additional files: {}", e),
    //         }
    //     }
    //     Ok(false) => return Ok(()),
    //     Err(e) => info!("Failed to prompt for additional files: {}", e),
    // }

    Ok(())
}

fn write_result(results: Vec<CleanupResult>) {
    use indicatif::{HumanBytes, HumanCount};

    let mut total_cleaned_size = 0u64;
    let mut total_missed_size = 0u64;
    let mut total_missed = 0u64;
    let mut total_cleaned = 0u64;
    for result in results {
        let cleaned_size = result.size();
        let missed_size = result.missed_size();
        let missed = result.missed_count();
        let cleaned = result.cleaned_count();
        total_cleaned_size += cleaned_size;
        total_missed_size += missed_size;
        total_missed += missed;
        total_cleaned += cleaned;

        match result {
            CleanupResult::Skipped { cleaner, reason } => info!("{cleaner} was skipped because {reason}"),
            CleanupResult::Failed { cleaner, source } => {
                error!("{cleaner} encountered an error during execution: {source}")
            }
            CleanupResult::Cleaned { cleaner, .. } => info!(
                "Cleanup removed {removed_files} files from {cleaner}, freeing up {removed_size}.",
                removed_files = HumanCount(cleaned),
                removed_size = HumanBytes(cleaned_size)
            ),
            CleanupResult::Partial { cleaner, .. } => {
                info!(
                    "Cleanup removed {removed_files} files from {cleaner}, freeing up {removed_size}.",
                    removed_files = HumanCount(cleaned),
                    removed_size = HumanBytes(cleaned_size)
                );

                info!(
                    "Automatic cleanup was unable to remove {missed_files} files from {cleaner}, which would have freed up an additional total of {missed_size}.",
                    missed_files = HumanCount(missed),
                    missed_size = HumanBytes(missed_size)
                )
            }
        };
    }

    if total_cleaned_size > 0 {
        info!(
            "Automatic cleanup removed a total of {removed_files} files, freeing up a total of {removed_size}.",
            removed_files = HumanCount(total_cleaned),
            removed_size = HumanBytes(total_cleaned_size)
        );
    }

    if total_missed > 0 {
        info!(
            "Automatic cleanup was unable to remove {missed_files}, which would have freed up an additional total of {missed_size}.",
            missed_files = HumanCount(total_missed),
            missed_size = HumanBytes(total_missed_size)
        )
    }
}

pub async fn run_cleaners(runtime: &'static Runtime) -> Vec<CleanupResult> {
    use std::time::Instant;

    let time = Instant::now();
    let mut results = vec![];

    let mut cleaner_stream = tokio_stream::iter(&runtime.cli.cleaners).map(|cleaner_ref| {
        let cleaner_ref = *cleaner_ref;
        let runtime = runtime;

        tokio::runtime::Handle::current().clone().spawn(async move {
            let cleaner_time = Instant::now();
            let cleaner = &*cleaner_ref;

            if !cleaner.supported() {
                return CleanupResult::Skipped {
                    cleaner: cleaner_ref,
                    reason: SkipReason::Unsupported,
                };
            }
            let result = cleaner.clean(runtime).await;
            debug!("Cleaner {cleaner:?} took {:?}", cleaner_time.elapsed());
            result
        })
    });

    while let Some(result) = cleaner_stream.next().await {
        match result.await {
            Ok(res) => results.push(res),
            Err(err) => error!("Cleaner encountered an error: {err}"),
        };
    }

    debug!("Completed cleanup in {:?}", time.elapsed());

    results
}
