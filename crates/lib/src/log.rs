/*
 * Copyright (C) 2024. James Draycott me@racci.dev
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
 * You should have received a copy of the GNU General Public License
 * along with this program. If not, see https://www.gnu.org/licenses/.
 */

use crate::ui::cli::flags::CommonFlags;
use std::env;
use tracing::{subscriber, Level, Subscriber};
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::fmt::format::FmtSpan;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::registry::LookupSpan;
use tracing_subscriber::Registry;

#[allow(dead_code)]
fn level_and_span(flags: &CommonFlags) -> (Level, FmtSpan) {
    match flags.verbose {
        0 => (Level::INFO, FmtSpan::NONE),
        1 => (Level::DEBUG, FmtSpan::NONE),
        2 => (Level::TRACE, FmtSpan::NONE),
        3 => (Level::TRACE, FmtSpan::ACTIVE),
        _ => (Level::TRACE, FmtSpan::FULL),
    }
}

#[inline]
fn add_file_writer<S>(
    registry: S,
) -> (
    impl Subscriber + for<'span> LookupSpan<'span> + Send + Sync + 'static,
    WorkerGuard,
)
    where
        S: Subscriber + for<'span> LookupSpan<'span> + Send + Sync + 'static,
{
    let file_appender = tracing_appender::rolling::daily(env::temp_dir().join("logs"), env!["CARGO_PKG_NAME"]);
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

    (
        registry.with(tracing_subscriber::fmt::layer().with_writer(non_blocking)),
        _guard,
    )
}

#[cfg(feature = "ui-cli")]
fn add_ui_layer<S>(
    registry: S,
    flags: &CommonFlags,
) -> impl Subscriber + for<'span> LookupSpan<'span> + Send + Sync + 'static
    where
        S: Subscriber + for<'span> LookupSpan<'span> + Send + Sync + 'static,
{
    use indicatif::ProgressStyle;
    use tracing_indicatif::{filter::IndicatifFilter, IndicatifLayer};
    use tracing_subscriber::{fmt::writer::MakeWriterExt, Layer};

    let (level, span) = level_and_span(flags);
    let layer = IndicatifLayer::new().with_progress_style(
        ProgressStyle::with_template(
            "{spinner:.green} {span_child_prefix}{span_name:.cyan/blue}{{{span_fields:.purple}}}",
        )
            .unwrap(),
    );

    let verbosity = if flags.quiet { 0 } else { flags.verbose };
    let quiet = flags.quiet;
    registry
        .with(
            tracing_subscriber::fmt::layer()
                .with_writer(
                    layer
                        .get_stderr_writer()
                        .with_filter(move |meta| match quiet {
                            true => meta.level() != &Level::WARN,
                            false => true,
                        } && meta.level() <= &level)
                        .with_max_level(Level::WARN)
                        .or_else(layer.get_stdout_writer().with_max_level(level)),
                )
                .without_time()
                .with_ansi(!cfg!(windows))
                .with_span_events(span)
                .with_target(verbosity > 0)
                .with_line_number(verbosity > 1)
                .with_thread_names(verbosity > 2)
                .with_thread_ids(verbosity > 2),
        )
        .with(layer.with_filter(IndicatifFilter::new(!flags.quiet)))
}

#[cfg(not(feature = "ui-cli"))]
fn add_ui_layer<S>(
    registry: S,
    _flags: &CommonFlags,
) -> impl Subscriber + for<'span> LookupSpan<'span> + Send + Sync + 'static
    where
        S: Subscriber + for<'span> LookupSpan<'span> + Send + Sync + 'static,
{
    registry
}

#[inline]
pub fn init(_named: &str, flags: &CommonFlags) -> WorkerGuard {
    let registry = Registry::default();
    let registry = add_ui_layer(registry, flags);
    let (registry, _guard) = add_file_writer(registry);

    subscriber::set_global_default(registry).expect("Failed to set global default subscriber");
    _guard
}
