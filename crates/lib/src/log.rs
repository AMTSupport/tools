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

use crate::cli::Flags;
use anyhow::Context;
use tracing::debug;
use tracing_subscriber::fmt;

pub fn init(_named: &str, cli: &Flags) -> anyhow::Result<()> {
    let level = match cli.verbose {
        0 => tracing::Level::INFO,
        1 => tracing::Level::DEBUG,
        _ => tracing::Level::TRACE,
    };

    tracing::subscriber::set_global_default(
        fmt::Subscriber::builder()
            .with_max_level(level)
            .pretty()
            .finish()
            // .with(fmt::Layer::default().with_writer(std::io::stdout)),
    )
    .context("Set global default logger")?;
    debug!("Initialised global logger");

    Ok(())
}

// #[cfg(windows)]
// fn platform_logger(named: &str) -> tracing_subscriber::Layer<_> {
//     let dir = temp_dir().join("amt");
//     let name_prefix = format!("log-{}", named);
//     let file_writer = tracing_appender::rolling::daily(dir, name_prefix);
//     let (file_appender, _guard) = tracing_appender::non_blocking(file_writer);
//
//     file_appender
// }
//
// fn platform_logger(named: &str) -> tracing_journald::Layer {
//     tracing_journald::layer().unwrap().with_syslog_identifier(named.to_string())
// }
