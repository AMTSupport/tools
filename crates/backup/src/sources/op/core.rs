use crate::config::backend::Backend;
use crate::config::backend::Backend::OnePassword;
use crate::config::runtime::RuntimeConfig;
use crate::sources::auto_prune::Prune;
use crate::sources::downloader::Downloader;
use crate::sources::exporter::Exporter;
use crate::sources::interactive::Interactive;
use crate::sources::op::account;
use crate::sources::op::account::OnePasswordAccount;
use crate::sources::op::one_pux;
use async_trait::async_trait;
use const_format::formatcp;
use indicatif::{MultiProgress, ProgressBar};
use inquire::Select;
use lib::anyhow::{anyhow, Context};
use lib::anyhow::Result;
use lib::fs::normalise_path;
use serde::{Deserialize, Serialize};
use serde_json::to_string_pretty;
use std::fmt::{Display, Formatter};
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;
use std::{env, fs};
use zip::write::FileOptions;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OnePasswordCore {
    pub account: OnePasswordAccount,
}

impl OnePasswordCore {
    pub fn data_dir(config: &RuntimeConfig) -> PathBuf {
        Self::base_dir(config).join("data")
    }
}

#[derive(Debug)]
enum AccountType {
    Service,
    Personal,
}

impl Display for AccountType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            AccountType::Service => "Service",
            AccountType::Personal => "Personal",
        };

        write!(f, "{}", str)
    }
}

impl Downloader for OnePasswordCore {
    const BINARY: &'static str = if cfg!(windows) { "op.exe" } else { "op" };
    const URL: &'static str = formatcp!(
        "https://cache.agilebits.com/dist/1P/op2/pkg/{version}/op_{os}_{arch}_{version}.zip",
        version = "v2.18.0",
        os = env::consts::OS,
        arch = if cfg!(target_arch = "x86") {
            "386"
        } else if cfg!(target_arch = "x86_64") {
            "amd64"
        } else {
            panic!("Unsupported arch")
        }
    );

    fn base_command(config: &RuntimeConfig) -> Command {
        let mut command = Command::new(Self::binary(config));
        command
            .arg("--cache")
            .args(["--config", Self::data_dir(config).display().to_string().as_str()]);

        command
    }
}

#[async_trait]
impl Exporter for OnePasswordCore {
    const DIRECTORY: &'static str = "1Password";

    async fn interactive(config: &RuntimeConfig) -> Result<Vec<Backend>> {
        let account_type = Select::new(
            "Which type of Account Accessor do you want to setup.",
            vec![AccountType::Personal, AccountType::Service]
        ).with_help_message(r#"
            A Service Account is a special type of account which can be logged in with a single token, however it cannot access Personal Vaults.
            A Personal Account is the standard way of authenticating with the cli which requires the 1Password desktop application to be installed,
            When using a Personal Account please ensure that the 1Password Desktop app doesn't have cli integration enabled.
        "#.trim()).prompt()?;

        let account = match account_type {
            AccountType::Personal => account::PersonalAccount::interactive(config).await,
            AccountType::Service => account::ServiceAccount::interactive(config).await,
        }?;

        Ok(vec![OnePassword(OnePasswordCore { account })])
    }

    // TODO :: Export of extra stuff like logos in the zip
    // TODO :: I'm unsure if that's even possible though.
    /// Creates a 1PUX compatible export,
    ///
    /// The name of this file is in format of "1Password-{uuid of the account exporting it}-{%Y%m%d-%H%M%S}.1pux"
    async fn export(
        &mut self,
        config: &RuntimeConfig,
        main_bar: &ProgressBar,
        progress_bar: &MultiProgress,
    ) -> Result<()> {
        let account = self.account.get();
        let (export, errors) = one_pux::export::Export::from(account, config, (main_bar, progress_bar))?;

        let file = self.account.get().directory(config).join(export.name);
        let file = normalise_path(file);

        account.ensure_directory(config)?;

        let file = fs::File::create_new(file).context("Create export file")?;
        let mut zip = zip::ZipWriter::new(file);

        let options = FileOptions::default();
        let serialised = to_string_pretty(&export.attributes).context("Serialise to 1PUX")?;
        zip.start_file("export.attributes", options).context("Start writer for attrs.")?;
        zip.write_all(serialised.as_bytes()).context("Write attrs to zip file.")?;

        let serialised = to_string_pretty(&export.data).context("Serialise to 1PUX")?;
        zip.start_file("export.data", options)?;
        zip.write_all(serialised.as_bytes())?;

        zip.add_directory("files", options).context("Create file directory")?;
        for file in export.files {
            zip.start_file(format!("files/{}", file.name), options)?;
            zip.write_all(&file.data)?;
        }

        zip.finish().context("Finish export file")?;

        if errors.len() > 0 {
            return Err(anyhow!("Errors occurred during export: {:?}", errors));
        }

        return Ok(());
    }
}

impl Prune for OnePasswordCore {
    fn files(&self, config: &RuntimeConfig) -> Vec<PathBuf> {
        let dir = self.account.get().directory(config).join("export_*.zip");
        let dir = normalise_path(dir);
        let glob = glob::glob(dir.to_str().unwrap()).unwrap();
        glob.flatten().collect()
    }
}
