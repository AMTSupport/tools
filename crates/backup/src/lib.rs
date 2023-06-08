#![feature(path_file_prefix)]
#![feature(type_alias_impl_trait)]
#![feature(trait_alias)]
#![feature(exit_status_error)]

use chrono::{DateTime, FixedOffset, Utc};
use lib::anyhow;
use lib::anyhow::{anyhow, Context};
use lib::simplelog::{trace, warn};
use std::path::PathBuf;

pub mod application;
pub mod config;
pub mod sources;

fn continue_loop<I>(vec: &Vec<I>, prompt_type: &str) -> bool {
    if vec.is_empty() {
        return true;
    }

    let should_continue =
        inquire::Confirm::new(&*format!("Do you want to add another {}?", prompt_type))
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

fn env_or_prompt(
    key: &str,
    prompt_title: &str,
    sensitive_info: bool,
    interactive: &bool,
    validator: fn(&String) -> bool,
) -> anyhow::Result<String> {
    let value = std::env::var(key);
    if let Ok(value) = value {
        if !validator(&value) {
            return Err(anyhow!("{} is not valid", key))?;
        }

        return Ok(value);
    }

    if !interactive {
        return Err(anyhow!(
            "{} is not set and interactive mode is disabled",
            key
        ))?;
    }

    let prompt = match sensitive_info {
        true => inquire::Password::new(prompt_title).prompt(),
        false => inquire::Text::new(prompt_title).prompt(),
    };

    match prompt {
        Ok(value) => Ok(value),
        Err(err) => Err(anyhow!("Failed to get {} from user: {}", key, err))?,
    }
}
