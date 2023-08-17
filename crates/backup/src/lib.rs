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

#![feature(path_file_prefix)]
#![feature(type_alias_impl_trait)]
#![feature(trait_alias)]
#![feature(exit_status_error)]
#![feature(unwrap_infallible)]
#![feature(slice_pattern)]
#![feature(let_chains)]
#![feature(return_position_impl_trait_in_trait)]
#![feature(result_option_inspect)]
#![feature(thin_box)]
#![feature(async_closure)]
#![feature(file_create_new)]
#![feature(const_trait_impl)]
#![feature(lazy_cell)]
#![feature(result_flattening)]

extern crate core;

use inquire::validator::StringValidator;
use lib::anyhow::{anyhow, Context, Result};
use tracing::{trace, warn};

pub mod application;
pub mod config;
pub mod sources;

fn continue_loop<I>(vec: &Vec<I>, prompt_type: &str) -> bool {
    if vec.is_empty() {
        return true;
    }

    let should_continue =
        inquire::Confirm::new(&format!("Do you want to add another {}?", prompt_type))
            .with_default(true)
            .prompt()
            .with_context(|| format!("Prompting for additional {}", prompt_type));

    match should_continue {
        Ok(should_continue) => should_continue,
        Err(err) => {
            warn!(
                "Failed to get confirmation for additional {}: {}",
                prompt_type, err
            );
            false
        }
    }
}

// TODO:: Derive title from key
fn env_or_prompt<V>(key: &str, validator: V) -> Result<String>
where
    V: StringValidator + 'static,
{
    match std::env::var(key) {
        Ok(str) => match validator.validate(&str) {
            Err(err) => Err(anyhow!("{} is set but invalid: {}", key, err)),
            Ok(_) => {
                trace!("Validated {} from env", key);
                Ok(str)
            }
        },
        _ => match inquire::Text::new(key) // TODO :: Pretty title
            .with_validator(validator)
            .prompt()
        {
            Err(err) => Err(anyhow!("Failed to get {} from user: {}", key, err)),
            Ok(str) => {
                trace!("Validated {} from user", key);
                Ok(str)
            }
        },
    }
}
