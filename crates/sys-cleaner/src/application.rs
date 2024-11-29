/*
 * Copyright (c) 2024. James Draycott <me@racci.dev>
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, version 3.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.
 * See the GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License along with this program.
 * If not, see <https://www.gnu.org/licenses/>.
 */

use crate::cleaners::cleaner::{CleanupResult, SkipReason};
use crate::config::runtime::Runtime;
use tokio_stream::StreamExt;
use tracing::{debug, error, info};

pub async fn application(runtime: &'static Runtime) -> anyhow::Result<()> {
    let results = run_cleaners(runtime).await;
    write_result(results);
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
            CleanupResult::Skipped { cleaner, reason } => info!("{cleaner} was skipped due to {reason}"),
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
                    r#"
                    Automatic cleanup was unable to remove {missed_files} files from {cleaner}.
                    There is an additional {missed_size} that could not be cleaned.
                    "#,
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
