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

use anyhow::Context;
use fmt::Subscriber;
use tracing::{debug, subscriber, Level};
use tracing_subscriber::fmt;

pub fn init(_named: &str, verbosity: u8) -> anyhow::Result<()> {
    let level = match verbosity {
        0 => Level::INFO,
        1 => Level::DEBUG,
        _ => Level::TRACE,
    };

    let builder = Subscriber::builder().with_max_level(level).pretty().without_time().compact();

    subscriber::set_global_default(builder.finish())
        .with_context(|| "Set global default logger")
        .inspect(|_| debug!("Initialised global logger"))
}
