use crate::config::runtime::RuntimeConfig;
use crate::sources::downloader::Downloader;
use crate::sources::exporter::Exporter;
use crate::sources::interactive::Interactive;
use crate::sources::op::cli;
use crate::sources::op::core::OnePasswordCore;
use async_trait::async_trait;
use inquire::list_option::ListOption;
use inquire::validator::Validation;
use inquire::{MultiSelect, Text};
use lib::anyhow::{anyhow, Context, Result};
use lib::fs::normalise_path;
use serde::{Deserialize, Serialize};
use serde_json::from_slice;
use std::fmt::{Debug, Display, Formatter};
use std::fs;
use std::path::PathBuf;
use std::process::Command;

#[cfg(unix)]
use std::os::unix::prelude::PermissionsExt;

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
            Self::Service(account) => write!(
                f,
                "Service Account: {}-{}",
                &account.attrs.name, &account.attrs.id
            ),
            Self::Personal(account) => write!(
                f,
                "Personal Account: {}-{}",
                &account.attrs.email, &account.attrs.id
            ), // TODO :: Domain
        }
    }
}

#[async_trait]
pub trait AccountCommon
where
    Self: Display + Debug + Send + Sync + 'static,
{
    /// Signs into the account and returns whether or not the signin was successful.
    async fn signin(&self, config: &RuntimeConfig) -> bool;

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

    fn vaults(&self) -> Vec<Vault>;

    fn account(&self) -> &cli::account::Account;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceAccount {
    pub attrs: cli::account::Account,
    pub token: String,
    pub vaults: Vec<Vault>,
}

impl ServiceAccount
where
    Self: AccountCommon,
{
    fn new(attrs: cli::account::Account, token: String, config: &RuntimeConfig) -> Result<Self> {
        let instance = Self {
            attrs,
            token,
            vaults: vec![],
        };
        instance.ensure_directory(config)?;
        Ok(instance)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonalAccount {
    pub attrs: cli::account::Account,
    // pub url: String,
    // pub email: String,
    // pub user_uid: String,
    pub vaults: Vec<Vault>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vault {
    pub id: String,
    pub name: String,
}

impl Display for Vault {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}-{}", &self.name, &self.id)
    }
}

#[async_trait]
impl Interactive<OnePasswordAccount> for ServiceAccount {
    async fn interactive(config: &RuntimeConfig) -> Result<OnePasswordAccount> {
        // TODO :: Validator
        let token = Text::new("Enter your 1Password service token")
            .with_help_message(
                "You can setup a service token at https://my.1password.com/integrations/infrastructure-secrets",
            )
            .prompt()?;

        let output = &Command::new(OnePasswordCore::binary(config))
            .env("OP_SERVICE_ACCOUNT_TOKEN", &token)
            .args(["user", "get", "--me", "--format=json"])
            .output()?;

        // This check should be redundant, but it's here just in case.
        if !output.status.success() {
            return Err(anyhow!(
                r#"
                Failed to get account information from 1Password
                (status code: {0})
                "#,
                output.status
            ));
        }

        let mut account = from_slice::<cli::account::Account>(&output.stdout)
            .context("Failed to parse account as Service Account, this could mean your token is invalid.")
            .and_then(|attrs| ServiceAccount::new(attrs, token, config))
            .inspect(|account| info!("Signed into 1Password as {}", account.attrs.name))?;

        let vaults = &Command::new(OnePasswordCore::binary(config))
            .env("OP_SERVICE_ACCOUNT_TOKEN", &account.token)
            .args(["vault", "list", "--format=json"])
            .output()?;
        let vaults = from_slice::<Vec<Vault>>(&vaults.stdout)?;

        account.vaults = MultiSelect::new("Select the vaults you want to use", vaults)
            .with_validator(|selections: &[ListOption<&Vault>]| match selections.len() {
                0 => Ok(Validation::Invalid(
                    "You must select at least one vault.".into(),
                )),
                _ => Ok(Validation::Valid),
            })
            .prompt()?;

        Ok(OnePasswordAccount::Service(account))
    }
}

#[async_trait]
impl Interactive<OnePasswordAccount> for PersonalAccount {
    // TODO :: Error Handling
    // TODO :: Cli integration instance
    async fn interactive(_config: &RuntimeConfig) -> Result<OnePasswordAccount> {
        return Err(anyhow!("Personal accounts are not yet supported."));

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
    async fn signin(&self, _config: &RuntimeConfig) -> bool {
        true
    }

    fn command(&self, config: &RuntimeConfig) -> Command {
        let directory = self.directory(config);
        let mut command = Command::new(OnePasswordCore::binary(config));
        command.args(["--config", &directory.display().to_string()]);
        command.arg("--cache");
        command.env("OP_SERVICE_ACCOUNT_TOKEN", &self.token);
        command
    }

    fn directory(&self, config: &RuntimeConfig) -> PathBuf {
        let dir = OnePasswordCore::base_dir(config).join(format!(
            "{name}-{id}",
            name = &self.attrs.name,
            id = &self.attrs.id
        ));

        normalise_path(dir)
    }

    fn vaults(&self) -> Vec<Vault> {
        self.vaults.clone()
    }

    fn account(&self) -> &cli::account::Account {
        &self.attrs
    }
}

#[async_trait]
impl AccountCommon for PersonalAccount {
    async fn signin(&self, config: &RuntimeConfig) -> bool {
        self.command(config)
            .args(["signin", "--account", &self.attrs.id])
            .output()
            .is_ok_and(|out| out.status.success())
    }

    fn command(&self, config: &RuntimeConfig) -> Command {
        let directory = self.directory(config);
        let mut command = Command::new(OnePasswordCore::binary(config));
        command.args(["--config", &directory.display().to_string()]);
        command
    }

    // TODO :: Ensure this is a valid directory name on windows
    fn directory(&self, config: &RuntimeConfig) -> PathBuf {
        let dir = OnePasswordCore::base_dir(config).join(format!(
            "{email}-{domain}",
            email = &self.attrs.email,
            domain = &self
                .attrs
                .id
                .split('.')
                .next()
                .map(|s| s.strip_prefix("https://").unwrap())
                .unwrap()
        ));

        normalise_path(dir)
    }

    fn vaults(&self) -> Vec<Vault> {
        self.vaults.clone()
    }

    fn account(&self) -> &cli::account::Account {
        &self.attrs
    }
}

impl Display for ServiceAccount {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.attrs.name)
    }
}

impl Display for PersonalAccount {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{email} - {domain}",
            email = &self.attrs.email,
            domain = &self.attrs.id // TODO :: Domain
        )
    }
}
