use crate::config::backend::Backend;
use crate::config::backend::Backend::OnePassword;
use crate::config::runtime::RuntimeConfig;
use crate::sources::auto_prune::Prune;
use crate::sources::download;
use crate::sources::exporter::Exporter;
use crate::sources::interactive::Interactive;
use crate::sources::op::account;
use crate::sources::op::account::OnePasswordAccount;
use async_trait::async_trait;
use const_format::formatcp;
use futures_util::StreamExt;
use inquire::Select;
use lib::anyhow::Context;
use lib::anyhow::Result;
use lib::simplelog::{debug, trace};
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::path::PathBuf;
use std::{env, fs};

use chrono::Local;
#[cfg(unix)]
use std::os::unix::prelude::PermissionsExt;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OnePasswordCore {
    pub account: OnePasswordAccount,
}

impl OnePasswordCore {
    #[cfg(unix)]
    const FILENAME: &'static str = "op";
    #[cfg(windows)]
    const FILENAME: &'static str = "op.exe";
    const DIRECTORY: &'static str = "1Password";
    // const CLI_VERSION: &'static str = "v2.19.0-beta.01";
    const CLI_VERSION: &'static str = "v2.18.0";
    #[cfg(target_arch = "x86")]
    const ARCH: &'static str = "386";
    #[cfg(target_arch = "x86_64")]
    const ARCH: &'static str = "amd64";
    const URL: &'static str = formatcp!(
        "https://cache.agilebits.com/dist/1P/op2/pkg/{version}/op_{os}_{arch}_{version}.zip",
        version = OnePasswordCore::CLI_VERSION,
        os = env::consts::OS,
        arch = OnePasswordCore::ARCH
    );

    pub fn binary(config: &RuntimeConfig) -> PathBuf {
        Self::base_dir(config).join(Self::FILENAME)
    }

    pub fn base_dir(config: &RuntimeConfig) -> PathBuf {
        config.directory.join(Self::DIRECTORY)
    }

    pub fn data_dir(config: &RuntimeConfig) -> PathBuf {
        Self::base_dir(&config).join("data")
    }

    async fn download_cli(config: &RuntimeConfig) -> Result<String> {
        let target = Self::binary(&config);
        // TODO :: Check for correct version, platform & arch
        if target.exists() && target.is_file() {
            trace!("Using existing CLI binary: {}", target.to_str().unwrap());
            return Ok(target.to_str().unwrap().to_string());
        }

        debug!("Downloading CLI binary from {} to {}", Self::URL, &target.display());
        let response = reqwest::Client::new().get(Self::URL).send().await?;
        if !response.status().is_success() {
            return Err(lib::anyhow::anyhow!("Failed to download CLI: {}", response.status()));
        }

        let total_size = response.content_length().unwrap();
        let stream = response.bytes_stream().boxed();
        let download = download(total_size, stream).await?;
        let file = fs::File::open(&download).context("Open Download File")?;
        let mut archive = zip::ZipArchive::new(file).context("Open Zip Archive")?;
        let mut found = false;

        // TODO :: Generic function for finding file in archive
        for i in 0..archive.len() {
            let mut file = archive.by_index(i).context("Get file by index")?;
            if file.is_file() && file.name() == target.file_name().unwrap() {
                fs::create_dir_all(&target.parent().unwrap()).context("Create parent directory for CLI binary")?;

                let mut out = fs::File::create(&target).context("Create file for CLI binary")?;
                std::io::copy(&mut file, &mut out).context("Copy CLI binary from archive to file")?;

                found = true;
                break;
            }
        }

        if !found {
            return Err(lib::anyhow::anyhow!("Failed to find CLI binary in archive"));
        }

        // TODO :: Windows permissions
        #[cfg(unix)]
        let mut permissions = fs::metadata(&target).context("Get metadata for CLI binary")?.permissions();
        #[cfg(unix)]
        permissions.set_mode(0o755);
        #[cfg(unix)]
        fs::set_permissions(&target, permissions).context("Set permissions for CLI binary")?;

        Ok(target.to_str().unwrap().to_string())
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

#[async_trait]
impl Exporter for OnePasswordCore {
    async fn interactive(config: &RuntimeConfig) -> Result<Vec<Backend>> {
        OnePasswordCore::download_cli(&config).await?;
        let account_type = Select::new(
            "Which type of Account Accessor do you want to setup.",
            vec![AccountType::Personal, AccountType::Service]
        ).with_help_message(r#"
            A Service Account is a special type of account which can be logged in with a single token, however it cannot access Personal Vaults.
            A Personal Account is the standard way of authenticating with the cli which requires the 1Password desktop application to be installed,
            When using a Personal Account please ensure that the 1Password Desktop app doesn't have cli integration enabled.
        "#.trim()).prompt()?;

        let account = match account_type {
            AccountType::Personal => account::PersonalAccount::interactive(&config).await,
            AccountType::Service => account::ServiceAccount::interactive(&config).await,
        }?;

        Ok(vec![OnePassword(OnePasswordCore { account })])
    }

    // TODO :: Zip export
    async fn export(
        &mut self,
        config: &RuntimeConfig,
        _main_bar: &ProgressBar,
        _progress_bar: &MultiProgress,
    ) -> Result<()> {
        let export = one_pux::create_export(Box::new(self.account.get()), &config);

        let file = format!("export_{}.json", Local::now().format("%Y-%m-%dT%H-%M-%SZ%z"));
        let file = self.account.get().directory(&config).join(file);
        let writer = fs::File::create(&file).context("Create export file")?;
        serde_json::to_writer_pretty(writer, &export.await?).context("Write export file")?;

        return Ok(());
    }
}

impl Prune for OnePasswordCore {
    // TODO
    fn files(&self, _config: &RuntimeConfig) -> Vec<PathBuf> {
        vec![]
    }
}
