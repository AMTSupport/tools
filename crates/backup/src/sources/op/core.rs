use crate::sources::downloader::Downloader;
use crate::{
    config::{
        backend::{Backend, Backend::OnePassword},
        runtime::RuntimeConfig,
    },
    sources::{
        auto_prune::Prune,
        exporter::Exporter,
        interactive::Interactive,
        op::{account, account::OnePasswordAccount, one_pux},
    },
};
use async_trait::async_trait;
use chrono::Local;
use const_format::formatcp;
use indicatif::{MultiProgress, ProgressBar};
use inquire::Select;
use lib::anyhow::Context;
use lib::anyhow::Result;
use lib::fs::normalise_path;
use serde::{Deserialize, Serialize};
use serde_json::to_string_pretty;
use std::fmt::{Display, Formatter};
use std::io::Write;
use std::path::PathBuf;
use std::{env, fs};
use zip::write::FileOptions;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OnePasswordCore {
    pub account: OnePasswordAccount,
}

impl OnePasswordCore {
    pub fn data_dir(config: &RuntimeConfig) -> PathBuf {
        Self::base_dir(&config).join("data")
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
    fn files(&self, config: &RuntimeConfig) -> Vec<PathBuf> {
        let dir = self.account.get().directory(&config).join("export_*.zip");
        let dir = normalise_path(dir);
        let glob = glob::glob(dir.to_str().unwrap()).unwrap();
        glob.flatten().collect()
    }
}
