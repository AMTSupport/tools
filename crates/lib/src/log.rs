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

use tracing::dispatcher::DefaultGuard;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::fmt::format::FmtSpan;
use tracing_subscriber::util::SubscriberInitExt;

pub fn init(_named: &str, verbosity: u8) -> DefaultGuard {
    let (level, span) = match verbosity {
        0 => (LevelFilter::INFO, FmtSpan::NONE),
        1 => (LevelFilter::DEBUG, FmtSpan::NONE),
        2 => (LevelFilter::TRACE, FmtSpan::NONE),
        _ => (LevelFilter::TRACE, FmtSpan::FULL),
    };

    tracing_subscriber::fmt()
        .without_time()
        .with_thread_names(verbosity > 2)
        .with_thread_ids(verbosity > 2)
        .with_level(verbosity > 0)
        .with_line_number(verbosity > 0)
        .with_max_level(level)
        .with_span_events(span.clone())
        .with_target(true)
        .finish()
        .init();

    tracing_subscriber::fmt()
        .without_time()
        .with_thread_names(verbosity > 2)
        .with_thread_ids(verbosity > 2)
        .with_level(verbosity > 0)
        .with_line_number(verbosity > 0)
        .with_max_level(level)
        .with_span_events(span)
        .with_target(true)
        .finish()
        .set_default()
}
