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

use crate::ui::cli::error::CliError;
use crate::ui::cli::oneshot::OneshotHandler;
use anyhow::{anyhow, Context, Result};
use inquire::validator::StringValidator;
use inquire::Text;
use std::fmt::Debug;
use tracing::{trace, warn};
use ui_inquire::STYLE;

pub mod error;
pub mod flags;
pub mod oneshot;
pub mod progress;
#[cfg(feature = "ui-repl")]
pub mod repl;
#[path = "inquire.rs"]
pub mod ui_inquire;

pub type CliResult<T> = Result<T, CliError>;

crate::feature_trait! {
    pub trait CliUi where {
        #[cfg(feature = "ui-repl")]
        where { Self: Debug + OneshotHandler + repl::ReplHandler },
        #[cfg(not(feature = "ui-repl"))]
        where { Self: Debug + OneshotHandler }
    } for {
        /// Run the CLI Application.
        ///
        /// This will be run by parsing the std::env::args() with [`MaybeRepl`]
        /// and then running the appropriate command.
        ///
        /// If the MaybeRepl has [`MaybeRepl::repl`] set to true,
        /// and there is also a [`MaybeRepl::oneshot`] command,
        /// then the oneshot command will be run once as a repl command.
        async fn run(&mut self) -> CliResult<()>
        where
            Self: Sized,
        {
            use clap::Parser;
            #[cfg(feature = "ui-repl")]
            use clap::CommandFactory;

            #[cfg(feature = "ui-repl")]
            let (command, mut factory) = (
                repl::ReplParser::<<Self as OneshotHandler>::OneshotAction>::parse(),
                <Self as repl::ReplHandler>::ReplAction::command()
            );
            #[cfg(not(feature = "ui-repl"))]
            let command = oneshot::OneshotParser::<<Self as OneshotHandler>::OneshotAction>::parse();

            #[cfg(feature = "updater")]
            let updater = if command.flags.update {
                Some(crate::updater::update())
            } else {
                None
            };

            #[cfg(feature = "ui-repl")]
            if command.repl {
                <Self as repl::ReplHandler>::repl(self).await?;
            } else if let Some(action) = command.action {
                <Self as OneshotHandler>::handle(self, action, &command.flags).await?;
            } else {
                factory.print_help().map_err(CliError::WriteError)?;
            }
            #[cfg(not(feature = "ui-repl"))]
            <Self as OneshotHandler>::handle(self, command.action, &command.flags).await?;

            #[cfg(feature = "updater")]
            // Wait for the update to finish before returning.
            if let Some(updater) = updater {
                tracing::info!("Waiting for update to finish...");
                match updater.await {
                    Ok(_) => tracing::info!("Update successful!"),
                    Err(err) => tracing::error!("Update failed: {err}"),
                }
            }

            Ok(())
        }
    }
}

pub fn continue_loop<I>(vec: &[I], prompt_type: &str) -> bool {
    if vec.is_empty() {
        return true;
    }

    let should_continue = inquire::Confirm::new(&format!("Do you want to add another {}?", prompt_type))
        .with_render_config(*STYLE)
        .with_default(true)
        .prompt()
        .with_context(|| format!("Prompting for additional {}", prompt_type));

    match should_continue {
        Ok(should_continue) => should_continue,
        Err(err) => {
            warn!("Failed to get confirmation for additional {}: {}", prompt_type, err);
            false
        }
    }
}

// TODO:: Derive title from key
pub fn env_or_prompt<V>(key: &str, validator: V) -> Result<String>
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
        _ => match Text::new(key) // TODO :: Pretty title
            .with_render_config(*STYLE)
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

#[macro_export]
macro_rules! populate {
    ($self:ident, $flags:ident) => {
        use amt_lib::log::init as _init;

        if $self._guard.is_none() {
            $self._guard = Some(_init(env!("CARGO_PKG_NAME"), $flags));
        }
    };
}

#[macro_export]
macro_rules! handler {
    ($vis:vis $name:ident $([ $($(#[$fmeta:meta])* $fvis:vis $fname:ident: $fty:ty)* ])? $({ $($item:item)* })?) => {
        $crate::handler!($vis $name<O> $([ $($fvis $fname $fty)* ])? $({ $($item)* })?);
    };
    ($vis:vis $name:ident<$ty:ty> $([ $($(#[$fmeta:meta])* $fvis:vis $fname:ident: $fty:ty)* ])? $({ $($item:item)* })?) => {
        use $crate::ui::cli::flags::CommonFlags as _CommonFlags;
        use $crate::ui::cli::CliResult as _CliResult;
        use clap::{Parser as _Parser, Subcommand as _Subcommand};
        use std::fmt::Debug as _Debug;

        paste::paste! {$vis trait [<$name Handler>] {
            type [<$name Action>]: _Debug + _Parser + _Subcommand;

            async fn handle(&mut self, command: Self::[<$name Action>], flags: &_CommonFlags) -> _CliResult<()>;

            $($($item)*)?
        }}

        paste::paste! {#[derive(_Debug, _Parser)] $vis struct [<$name Parser>]<O> where O: _Debug + _Parser + _Subcommand {
            #[clap(subcommand)]
            pub action: $ty,

            #[clap(flatten)]
            pub flags: _CommonFlags,

            $($($(#[$fmeta])* $fvis $fname: $fty,)*)?
        }}
    }
}
