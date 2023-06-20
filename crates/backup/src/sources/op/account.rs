use crate::config::runtime::RuntimeConfig;
use crate::sources::downloader::Downloader;
use crate::sources::exporter::Exporter;
use crate::sources::interactive::Interactive;
use crate::sources::op::cli;
use crate::sources::op::cli::CliGetter;
use crate::sources::op::core::OnePasswordCore;
use async_trait::async_trait;
use inquire::list_option::ListOption;
use inquire::validator::Validation;
use inquire::{MultiSelect, Text};
use lib::anyhow::{anyhow, Context, Result};
use lib::fs::normalise_path;
use serde::{Deserialize, Serialize};
use std::fmt::{Debug, Display, Formatter};
use std::fs;
use std::path::PathBuf;
use std::process::Command;

#[cfg(unix)]
use std::os::unix::prelude::PermissionsExt;
use futures_util::TryFutureExt;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OnePasswordAccount {
    Service(ServiceAccount),
    Personal(PersonalAccount),
}

impl OnePasswordAccount {
    pub fn get(&self) -> &dyn AccountCommon {
        match self {
            Self::Service(account) => account,
            Self::Personal(account) => account,
        }
    }
}

impl Display for OnePasswordAccount {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Service(account) => write!(f, "Service Account: {account}"),
            Self::Personal(account) => write!(f, "Personal Account: {account}"),
        }
    }
}

#[async_trait]
pub trait AccountCommon
where
    Self: Display + Debug + Send + Sync + 'static,
{
    /// Creates a new command with the required environment variables & arguments for the account.
    fn command(&self, config: &RuntimeConfig) -> Command;

    /// Returns the directory where the account's data is stored.
    /// This will be within the root 1Password directory with a unique name to identity the account.
    fn directory(&self, config: &RuntimeConfig) -> PathBuf;

    /// Ensures that the directory exists and has the correct permissions wanted by 1Password.
    /// 1Password requires that directories have 700 permissions. (Only the owner can read, write, and execute)
    fn ensure_directory(&self, config: &RuntimeConfig) -> Result<()> {
        let directory = self.directory(config);
        if !directory.exists() {
            fs::create_dir_all(&directory)?;
            #[cfg(unix)]
            fs::set_permissions(&directory, fs::Permissions::from_mode(0o700))?;
        }

        Ok(())
    }

    fn vaults(&self) -> &[cli::vault::Reference];

    fn account(&self) -> &cli::account::Fused;

    fn user(&self) -> &cli::user::User;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceAccount {
    pub user: cli::user::User,
    pub account: cli::account::Fused,
    pub token: String,
    pub vaults: Vec<cli::vault::Reference>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonalAccount {
    pub user: cli::user::User,
    pub account: cli::account::Fused,
    pub vaults: Vec<cli::vault::Reference>,
}

#[async_trait]
impl Interactive<OnePasswordAccount> for ServiceAccount {
    async fn interactive(config: &RuntimeConfig) -> Result<OnePasswordAccount> {
        // TODO :: Validator
        let token = Text::new("Enter your 1Password service token")
            .with_help_message(// TODO :: Wrong url
                "You can get a service token at https://my.1password.com/integrations/infrastructure-secrets",
            )
            .prompt()?;

        let envs: [(&str, &str); 1] = [("OP_SERVICE_ACCOUNT_TOKEN", &token)];

        let user = cli::user::User::get(config, &envs, &[]);
        let account = cli::account::Fused::get(config, &envs, &[]);

        let vaults = cli::vault::Reference::get(config, &envs, &[]).and_then(|v| async move {
            match v.len() {
                0 => Err(anyhow!("No vaults found for this account.")),
                1 => Ok(v),
                _ => MultiSelect::new("Select the vaults you want to use", v)
                    .with_validator(
                        |selections: &[ListOption<&cli::vault::Reference>]| match selections.len() {
                            0 => Ok(Validation::Invalid(
                                "You must select at least one vault.".into(),
                            )),
                            _ => Ok(Validation::Valid),
                        },
                    )
                    .prompt()
                    .context("Get vaults from user selection"),
            }
        });

        let instance = Self {
            user: user.await?,
            account: account.await?,
            vaults: vaults.await?,
            token,
        };
        instance.ensure_directory(config)?;

        Ok(OnePasswordAccount::Service(instance))
    }
}

#[async_trait]
impl Interactive<OnePasswordAccount> for PersonalAccount {
    // TODO :: Error Handling
    // TODO :: Cli integration instance
    async fn interactive(_config: &RuntimeConfig) -> Result<OnePasswordAccount> {
        return Err(anyhow!("Personal accounts are not yet supported."));

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
    }
}

#[async_trait]
impl AccountCommon for ServiceAccount {
    fn command(&self, config: &RuntimeConfig) -> Command {
        let mut command = OnePasswordCore::base_command(config);
        command.env("OP_SERVICE_ACCOUNT_TOKEN", &self.token);
        command
    }

    fn directory(&self, config: &RuntimeConfig) -> PathBuf {
        let dir = OnePasswordCore::base_dir(config).join(format!("{self}"));
        normalise_path(dir)
    }

    fn vaults(&self) -> &[cli::vault::Reference] {
        &self.vaults
    }

    fn account(&self) -> &cli::account::Fused {
        &self.account
    }

    fn user(&self) -> &cli::user::User {
        &self.user
    }
}

#[async_trait]
impl AccountCommon for PersonalAccount {
    fn command(&self, config: &RuntimeConfig) -> Command {
        let mut command = OnePasswordCore::base_command(config);
        command.args(["--account", &self.account.long.id]);
        command
    }

    // TODO :: Ensure this is a valid directory name on windows
    fn directory(&self, config: &RuntimeConfig) -> PathBuf {
        let dir = OnePasswordCore::base_dir(config).join(format!("{self}"));
        normalise_path(dir)
    }

    fn vaults(&self) -> &[cli::vault::Reference] {
        &self.vaults
    }

    fn account(&self) -> &cli::account::Fused {
        &self.account
    }

    fn user(&self) -> &cli::user::User {
        &self.user
    }
}

impl Display for ServiceAccount {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{name}@{domain}",
            name = self.user.name,
            domain = self
                .account
                .whoami
                .url
                .strip_prefix("https://")
                .unwrap_or(&self.account.whoami.url)
        )
    }
}

impl Display for PersonalAccount {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{name}@{domain}",
            name = &self.user.name,
            domain = self
                .account
                .whoami
                .url
                .strip_prefix("https://")
                .unwrap_or(&self.account.whoami.url)
        )
    }
}
