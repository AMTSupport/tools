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

use std::env;
use std::io::stdout;
use tracing::{subscriber, Level};
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::fmt::format::FmtSpan;
use tracing_subscriber::fmt::writer::MakeWriterExt;
use tracing_subscriber::fmt::Layer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::Registry;

#[inline]
pub fn init(named: &str, verbosity: u8) -> WorkerGuard {
    let (level, span) = match verbosity {
        0 => (Level::INFO, FmtSpan::NONE),
        1 => (Level::DEBUG, FmtSpan::NONE),
        2 => (Level::TRACE, FmtSpan::NONE),
        3 => (Level::TRACE, FmtSpan::ACTIVE),
        _ => (Level::TRACE, FmtSpan::FULL),
    };

    let file_appender = tracing_appender::rolling::daily(env::temp_dir().join("logs"), named);
    let (file_writer, guard) = tracing_appender::non_blocking(file_appender);

    let registry = Registry::default().with(Layer::default().with_writer(file_writer)).with(
        Layer::default()
            .without_time()
            .with_span_events(span)
            .with_writer(stdout.with_max_level(level))
            .with_target(verbosity > 0)
            .with_line_number(verbosity > 1)
            .with_thread_names(verbosity > 2)
            .with_thread_ids(verbosity > 2),
    );
    subscriber::set_global_default(registry).expect("Failed to set global default subscriber");

    guard
}
