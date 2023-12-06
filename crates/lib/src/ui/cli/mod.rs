/*
 * Copyright (c) 2023. James Draycott <me@racci.dev>
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, version 3.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License along with this program.
 * If not, see <https://www.gnu.org/licenses/>.
 */

use crate::ui::cli::error::CliError;
use crate::ui::cli::oneshot::OneshotHandler;
use crate::ui::{UiBuildable, UiBuildableFiller};
use anyhow::{anyhow, Context, Result};
use inquire::validator::StringValidator;
use inquire::Text;
use std::fmt::Debug;
use tracing::{error, instrument, trace, warn};

pub mod error;
pub mod oneshot;
pub mod progress;
#[cfg(feature = "ui-repl")]
pub mod repl;
#[path = "inquire.rs"]
pub mod ui_inquire;

pub type CliResult<T> = Result<T, CliError>;

auto trait FeatureTrait {}

crate::feature_trait! {
    pub trait CliUi where {
        #[cfg(feature = "ui-repl")]
        where { Self: OneshotHandler + repl::ReplHandler },
        #[cfg(not(feature = "ui-repl"))]
        where { Self: OneshotHandler }
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
                repl::ReplParser::<<Self as OneshotHandler>::Action>::parse(),
                <Self as repl::ReplHandler>::Action::command()
            );
            #[cfg(not(feature = "ui-repl"))]
            let command = oneshot::OneshotParser::<<Self as OneshotHandler>::Action>::parse();

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

            Ok(())
        }
    }
}

impl<C> UiBuildableFiller for C
where
    C: CliUi,
{
    #[instrument(level = "TRACE", ret, err)]
    async fn fill<B: UiBuildable<V>, V: From<B> + Debug>() -> Result<V> {
        let mut builder = B::default();
        let mut required_values = B::REQUIRED_FIELDS.to_vec();
        let mut optional_values = B::OPTIONAL_FIELDS.to_vec();

        for env_filled in builder.filled_fields() {
            trace!("Field {env_filled} was filled from env");
            required_values.retain(|field| field != env_filled);
            optional_values.retain(|field| field != env_filled);
        }

        for field in required_values {
            let message = format!("Please enter the value for {field}");
            let prompt =
                Text::new(&message).with_help_message("This value is required").with_placeholder("Enter value here...");

            let value = prompt.prompt()?;
            builder.set_field(field, &value)?;
        }

        for field in optional_values {
            let message = format!("Please enter the value for {field}");
            let prompt = Text::new(&message).with_help_message("This value is optional").with_default("");

            let value = prompt.prompt()?;
            builder.set_field(field, &value)?;
        }

        builder.build()
    }

    #[instrument(level = "TRACE", ret, err)]
    async fn modify<B: UiBuildable<V>, V: From<B> + Debug>(mut builder: B) -> Result<V> {
        for field in B::REQUIRED_FIELDS {
            let current = builder.display(field);
            let message = format!("Please enter the value for {field}");
            let prompt = Text::new(&message)
                .with_help_message("This value is required")
                .with_placeholder("Enter value here...")
                .with_default(&current);

            match prompt.prompt() {
                Ok(value) => builder.set_field(field, &value)?,
                Err(err) => {
                    error!("Failed to prompt for field {field}: {err}");
                    error!("Using current value: {current}");
                }
            }
        }

        for field in B::OPTIONAL_FIELDS {
            let current = builder.display(field);
            let message = format!("Please enter the value for {field}");
            let prompt = Text::new(&message)
                .with_help_message("This value is optional")
                .with_placeholder("Enter value here...")
                .with_default(&current);

            match prompt.prompt() {
                Ok(value) => builder.set_field(field, &value)?,
                Err(err) => {
                    error!("Failed to prompt for field {field}: {err}");
                    error!("Using current value: {current}");
                }
            }
        }

        builder.build()
    }
}

pub fn continue_loop<I>(vec: &Vec<I>, prompt_type: &str) -> bool {
    if vec.is_empty() {
        return true;
    }

    let should_continue = inquire::Confirm::new(&format!("Do you want to add another {}?", prompt_type))
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
        use lib::log::init as _init;

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
        use $crate::cli::Flags as _CommonFlags;
        use $crate::ui::cli::CliResult as _CliResult;
        use clap::{Parser as _Parser, Subcommand as _Subcommand};
        use std::fmt::Debug as _Debug;
        use paste::paste as _paste;

        _paste! {$vis trait [<$name Handler>] {
            type Action: _Debug + _Parser + _Subcommand;

            async fn handle(&mut self, command: Self::Action, flags: &_CommonFlags) -> _CliResult<()>;

            $($($item)*)?
        }}

        _paste! {#[derive(_Debug, _Parser)] $vis struct [<$name Parser>]<O> where O: _Debug + _Parser + _Subcommand {
            #[clap(subcommand)]
            pub action: $ty,

            #[clap(flatten)]
            pub flags: _CommonFlags,

            $($($(#[$fmeta])* $fvis $fname: $fty,)*)?
        }}
    }
}
