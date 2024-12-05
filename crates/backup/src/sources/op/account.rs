/*
 * Copyright (C) 2023-2024. James Draycott me@racci.dev
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

use crate::config::runtime::Runtime;
use crate::sources::downloader::Downloader;
use crate::sources::getter::{CliGetter, CommandFiller};
use crate::sources::op::cli;
use crate::sources::op::core::OnePasswordCore;
use amt_lib::pathed::Pathed;
use anyhow::{anyhow, Context, Result};
use macros::CommonFields;
use serde::{Deserialize, Serialize};
use std::fmt::{Debug, Display, Formatter};
use std::process::Command;
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AccountAttrs {
    pub user: cli::user::User,
    pub account: cli::account::Account,
    pub vaults: Vec<cli::vault::Reference>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, CommonFields)]
pub enum OnePasswordAccount {
    Personal { attrs: AccountAttrs },
    Service { attrs: AccountAttrs, token: String },
}

impl Pathed<Runtime> for OnePasswordAccount {
    const NAME: &'static str = "1Password";
    const PERMISSIONS: u32 = 0o700;

    fn get_unique_name(&self) -> String {
        match self {
            Self::Personal { attrs, .. } => attrs,
            Self::Service { attrs, .. } => attrs,
        }
        .account
        .attrs()
        .identifier
        .to_string()
    }
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("api limit has been reached.")]
    ApiLimitReached,

    #[error("unable to parse json response -> {0}")]
    Json(#[from] serde_json::Error),

    #[error("unable to parse json response -> {0}")]
    Execution(#[from] std::io::Error),
}

impl OnePasswordAccount {
    /// Creates a new command with the required environment variables & arguments for the account.
    pub(crate) fn command(&self, config: &Runtime) -> Result<Command> {
        let mut command = OnePasswordCore::base_command(config)?;
        let (fill_args, fill_envs) = self.fill();
        command.args(fill_args);
        command.envs(fill_envs);

        Ok(command)
    }
}

impl Display for OnePasswordAccount {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let attrs = self.attrs();
        let attrs = attrs.account.attrs();

        write!(
            f,
            "{name}@{domain}.1password",
            name = attrs.identifier,
            domain = attrs.domain
        )
    }
}

impl CommandFiller for OnePasswordAccount {
    fn fill(&self) -> (Vec<&str>, Vec<(&str, &str)>) {
        let mut envs = vec![];
        let mut args = vec![];
        match self {
            Self::Service { token, .. } => {
                envs.push(("OP_SERVICE_ACCOUNT_TOKEN", token.as_str()));
            }
            Self::Personal { attrs } => {
                args.extend(["--account", attrs.account.attrs().identifier.id()]);
            }
        };

        (args, envs)
    }
}

// #[async_trait]
// pub trait AccountCommon
// where
//     Self: Send + Sync + 'static,
// {
//     /// Ensures that the directory exists and has the correct permissions wanted by 1Password.
//     /// 1Password requires that directories have 700 permissions. (Only the owner can read, write, and execute)
//     fn ensure_directory(&self, config: &RuntimeConfig) -> Result<()> {
//         let directory = self.directory(config);
//         if !directory.exists() {
//             fs::create_dir_all(&directory)?;
//             #[cfg(unix)]
//             fs::set_permissions(&directory, fs::Permissions::from_mode(0o700))?;
//         }
//
//         Ok(())
//     }
//
//     fn vaults(&self) -> &[cli::vault::Reference];
//
//     fn account(&self) -> &cli::account::Account;
//
//     fn user(&self) -> &cli::user::User;
// }

// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub struct ServiceAccount {
//     pub user: cli::user::User,
//     pub account: cli::account::Account,
//     pub token: String,
//     pub vaults: Vec<cli::vault::Reference>,
// }
//
// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub struct PersonalAccount {
//     pub user: cli::user::User,
//     pub account: cli::account::Account,
//     pub vaults: Vec<cli::vault::Reference>,
// }

// #[async_trait]
// impl Interactive<OnePasswordAccount> for PersonalAccount {
//     // TODO :: Error Handling
//     // TODO :: Cli integration instance
//     async fn interactive(_config: &RuntimeConfig) -> Result<OnePasswordAccount> {
//         return Err(anyhow!("Personal accounts are not yet supported."));

// self.command(config)
//     .args(["signin", "--account", &self.attrs.id])
//     .output()
//     .is_ok_and(|out| out.status.success())

// if false {
//     trace!("Getting list of accounts from 1Password");
//     let output = Command::new(OnePasswordCore::binary(config))
//         .args(["account", "list", "--format=json"])
//         .output()?;
//
//     let accounts = match output.status.success() {
//         true => output.stdout,
//         false => {
//             return Err(anyhow!(
//                 r#"
//             Failed to get account information from 1Password
//             (stderr: {0})
//             "#,
//                 String::from_utf8_lossy(output.stderr.as_slice())
//             ))
//         }
//     };
//
//     trace!("Got list of accounts from 1Password: {:?}", &accounts);
//     let accounts: Vec<PersonalAccount> =
//         from_slice(&accounts).context("Failed to parse accounts as Personal Accounts")?;
//
//     trace!("Prompting user to select an account");
//     let account = Select::new(
//         "Which account do you want to use?",
//         accounts
//     ).with_help_message("If you don't see your account here, you may need to login to the 1Password desktop application first.").prompt()?;
//     trace!("Prompted user to select an account: {:?}", &account);
//
//     return Ok(OnePasswordAccount::Personal(account));
// }
//
// let _domain = Text::new("Enter your 1Password account domain")
//     .with_help_message(
//         "This is the domain you use to login to 1Password, e.g. `https://my.1password.com`",
//     )
//     .with_default("https://my.1password.com")
//     // TODO :: Better Validator
//     .with_validator(|url: &str| match url.starts_with("https://") {
//         true => Ok(Validation::Valid),
//         false => Ok(Validation::Invalid(
//             "The URL must start with https://".into(),
//         )),
//     })
//     .prompt()?;
//
// let _email = Text::new("Enter your 1Password account email")
//     .with_help_message("This is the email you use to login to 1Password")
//     // TODO :: Better Validator
//     .with_validator(|email: &str| match email.contains('@') {
//         true => Ok(Validation::Valid),
//         false => Ok(Validation::Invalid("Invalid email address!".into())),
//     })
//     .prompt()?;
//
// let _secret_key = Password::new("Enter your 1Password secret key")
//     .without_confirmation()
//     .with_help_message("This is the secret key you use to login to 1Password")
//     .prompt()?;
//
// let _password = Password::new("Enter your 1Password account password")
//     .without_confirmation()
//     .with_help_message("This is the password you use to login to 1Password")
//     .prompt()?;
//
// let _output = Command::new(OnePasswordCore::binary(config));
//     }
// }

// impl Display for ServiceAccount {
//     fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
//         account_display(self, f)
//     }
// }
//
// impl Display for PersonalAccount {
//     fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
//         account_display(self, f)
//     }
// }
//
// fn account_display<A: AccountCommon>(account: &A, f: &mut Formatter<'_>) -> std::fmt::Result {
//     let attrs = account.account().get_attrs();
//     let domain = &attrs.domain;
//     let name = &attrs.identifier.named();
//
//     write!(f, "{name}@{domain}.1password")
// }
