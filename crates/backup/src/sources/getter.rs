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

use crate::config::runtime::Runtime;
use crate::sources::downloader::Downloader;
use serde::de::DeserializeOwned;
use std::fmt::Debug;
use thiserror::Error;
use tracing::{debug, error, error_span, instrument, trace};

#[derive(Error, Debug)]
pub enum Error {
    #[error("api limit has been reached.")]
    ApiLimitReached,

    #[error("unable to parse json response -> {0}")]
    Json(#[from] serde_json::Error),

    #[error("unable to parse json response -> {0}")]
    Execution(#[from] std::io::Error),
}

pub(super) trait CommandFiller: Debug + Send + Sync + 'static {
    /// # Returns
    /// A pair tuple with the first element being arguments to add to the command,
    /// and the second element being the environment variables pair tuples to add to the command.
    fn fill(&self) -> (Vec<&str>, Vec<(&str, &str)>);
}

// TODO -> Convert into derive macro
/// # Arguments
/// * `B` - The downloader to use.
/// * `T` - The type to parse the json response into.
/// * `A` - The type for the const args.
pub(super) trait CliGetter<B, T, A>
where
    B: Downloader,
    T: Debug + DeserializeOwned,
    A: IntoIterator<Item = &'static str>,
{
    const ARGS: A;

    #[instrument(ret, err)]
    async fn get<F: CommandFiller>(
        config: &Runtime,
        filler: &F,
        envs: &[(&str, &str)],
        args: &[&str],
    ) -> anyhow::Result<T> {
        let (filler_args, filler_envs) = filler.fill();
        let mut args = args.to_owned();
        let mut envs = envs.to_owned();
        args.extend(filler_args);
        envs.extend(filler_envs);
        Self::_get(config, envs.as_slice(), args.as_slice()).await
    }

    #[instrument(ret, err)]
    async fn _get(config: &Runtime, envs: &[(&str, &str)], args: &[&str]) -> anyhow::Result<T> {
        let mut command = B::base_command(config)?;
        command.arg("--format=json");
        command.args(Self::ARGS);
        command.args(args);
        command.envs(envs.to_owned());

        trace!("Executing command: {command:?}");

        let output = match command.output() {
            Ok(o) => o,
            Err(e) => {
                error_span!("Command Error").in_scope(|| {
                    error!("Error executing command: {command:?}");
                    error!("Expecting json value for type: {:?}", std::any::type_name::<T>());
                });

                return Err(Error::Execution(e).into());
            }
        };

        if !output.status.success() || output.stderr.len() > 0 {
            error_span!("Command Error").in_scope(|| {
                error!("Error executing command: {command:?}");
                error!("Exit Code: {}", output.status.code().unwrap_or(0));
                error!("Stdout: {}", String::from_utf8_lossy(&output.stdout));
                error!("Stderr: {}", String::from_utf8_lossy(&output.stderr));
            });

            return Err(Error::Execution(std::io::Error::new(std::io::ErrorKind::Other, "Command failed")).into());
        }

        let stdout = output.stdout;
        match serde_json::from_slice::<T>(stdout.as_slice()) {
            Ok(v) => Ok(v),
            Err(e) => {
                error_span!("Command Error").in_scope(|| {
                    error!("Error executing command: {command:?}");
                    error!("Unable to parse json response: {e:?}");
                    debug!("Raw: {}", String::from_utf8_lossy(&stdout));
                });

                Err(Error::Json(e).into())
            }
        }
    }
}
