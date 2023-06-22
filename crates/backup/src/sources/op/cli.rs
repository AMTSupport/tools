#![allow(dead_code)]

use crate::config::runtime::RuntimeConfig;
use crate::sources::downloader::Downloader;
use crate::sources::op::core::OnePasswordCore;
use async_trait::async_trait;
use lib::anyhow::Result;
use std::process::Command;

#[async_trait]
pub(super) trait CliGetter<T> {
    async fn get(config: &RuntimeConfig, envs: &[(&str, &str)], args: &[&str]) -> Result<T>;

    fn prepared(config: &RuntimeConfig, envs: &[(&str, &str)], args: &[&str]) -> Command {
        let mut command = OnePasswordCore::base_command(config);
        envs.into_iter().for_each(|(key, value)| {
            command.env(key, value);
        });
        command.args(args);
        command
    }
}

pub mod user {
    use crate::config::runtime::RuntimeConfig;
    use crate::sources::op::cli::CliGetter;
    use async_trait::async_trait;
    use chrono::{DateTime, FixedOffset};
    use lib::anyhow::{anyhow, Context, Result};
    use serde::{Deserialize, Serialize};
    use serde_json::from_slice;
    use tracing::trace;

    // TODO :: Add more types
    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[serde(rename_all = "SCREAMING_SNAKE_CASE")]
    pub enum Type {
        Member,
        ServiceAccount,
    }

    // TODO :: Add more states
    #[derive(Default, Debug, Clone, Serialize, Deserialize)]
    #[serde(rename_all = "SCREAMING_SNAKE_CASE")]
    pub enum State {
        #[default]
        Active,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct User {
        pub id: String,
        pub name: String,
        pub email: String,
        #[serde(rename = "type")]
        pub user_type: Type,
        pub state: State,
        pub created_at: DateTime<FixedOffset>,
        pub updated_at: DateTime<FixedOffset>,
        pub last_auth_at: DateTime<FixedOffset>,
    }

    #[async_trait]
    impl CliGetter<User> for User {
        async fn get(config: &RuntimeConfig, envs: &[(&str, &str)], args: &[&str]) -> Result<User> {
            let mut command = Self::prepared(config, envs, args);
            command.args(args).args(["user", "get", "--me", "--format=json"]);
            trace!("Running command: {:?}", command);
            let output = command.output().context("Failed to get user from cli")?;

            match output.status.success() {
                false => Err(anyhow!(
                    "Failed to get user from cli: {}",
                    String::from_utf8_lossy(&output.stderr)
                )),
                true => from_slice(&output.stdout)
                    .map_err(|e| anyhow!("Failed to parse user from cli: {}", e)),
            }
        }
    }
}

pub mod account {
    use crate::config::runtime::RuntimeConfig;
    use crate::sources::op::cli::CliGetter;
    use crate::sources::op::one_pux;
    use async_trait::async_trait;
    use chrono::{DateTime, FixedOffset};
    use lib::anyhow::{anyhow, Context, Result};
    use serde::{Deserialize, Serialize};
    use serde_json::from_slice;
    use tracing::trace;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[serde(rename_all = "SCREAMING_SNAKE_CASE")]
    pub enum Type {
        Individual,
        Business,
    }

    // TODO :: Add more states
    #[derive(Default, Debug, Clone, Serialize, Deserialize)]
    #[serde(rename_all = "SCREAMING_SNAKE_CASE")]
    pub enum State {
        #[default]
        Active,
    }

    /// This comes from the list of account gotten with `op list accounts`
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Short {
        pub url: String,
        pub email: String,
        pub user_uuid: String,
        pub account_uuid: String,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[serde(rename_all = "PascalCase")]
    pub struct WhoAmI {
        #[serde(rename = "URL")]
        pub url: String,
        pub service_account_type: String,
    }

    /// A long detailed account gotten with `op account get`
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Account {
        pub id: String,
        pub name: String,
        #[serde(rename = "type")]
        pub account_type: Type,
        pub state: State,
        pub created_at: DateTime<FixedOffset>,
    }

    /// A struct containing both the short and long account info
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Fused {
        pub whoami: WhoAmI,
        pub long: Account,
    }

    #[async_trait]
    impl CliGetter<Vec<Short>> for Short {
        async fn get(
            config: &RuntimeConfig,
            envs: &[(&str, &str)],
            args: &[&str],
        ) -> Result<Vec<Short>> {
            let mut command = Self::prepared(config, envs, args);
            command.args(args).args(["account", "list", "--format=json"]);
            trace!("Running command: {:?}", command);
            let output = command.output().context("Failed to get accounts list from cli")?;

            match output.status.success() {
                false => Err(anyhow!(
                    "Failed to get accounts list from cli output: {}",
                    String::from_utf8_lossy(&output.stderr)
                )),
                true => from_slice(&output.stdout)
                    .map_err(|e| anyhow!("Failed to parse accounts list from cli output: {}", e)),
            }
        }
    }

    #[async_trait]
    impl CliGetter<WhoAmI> for WhoAmI {
        async fn get(
            config: &RuntimeConfig,
            envs: &[(&str, &str)],
            args: &[&str],
        ) -> Result<WhoAmI> {
            let mut command = Self::prepared(config, envs, args);
            command.args(["whoami", "--format=json"]);
            trace!("Running command: {:?}", command);
            let output = command.output().context("Failed to get whoami from cli")?;

            match output.status.success() {
                false => Err(anyhow!(
                    "Failed to get whoami from cli output: {}",
                    String::from_utf8_lossy(&output.stderr)
                )),
                true => from_slice(&output.stdout)
                    .map_err(|e| anyhow!("Failed to parse whoami from cli output: {}", e)),
            }
        }
    }

    #[async_trait]
    impl CliGetter<Account> for Account {
        async fn get(
            config: &RuntimeConfig,
            envs: &[(&str, &str)],
            args: &[&str],
        ) -> Result<Account> {
            let mut command = Self::prepared(config, envs, args);
            command.args(["account", "get", "--format=json"]);
            trace!("Running command: {:?}", command);
            let output = command.output().context("Failed to get account from cli")?;

            match output.status.success() {
                false => Err(anyhow!(
                    "Failed to get account from cli output: {}",
                    String::from_utf8_lossy(&output.stderr)
                )),
                true => from_slice(&output.stdout)
                    .map_err(|e| anyhow!("Failed to parse account from cli output: {}", e)),
            }
        }
    }

    #[async_trait]
    impl CliGetter<Fused> for Fused {
        async fn get(
            config: &RuntimeConfig,
            envs: &[(&str, &str)],
            args: &[&str],
        ) -> Result<Fused> {
            let long = Account::get(config, envs, args);
            let whoami = WhoAmI::get(config, envs, args);

            Ok(Fused {
                whoami: whoami.await?,
                long: long.await?,
            })
        }
    }

    impl From<Fused> for one_pux::account::Attrs {
        fn from(value: Fused) -> one_pux::account::Attrs {
            one_pux::account::Attrs {
                account_name: value.long.name.clone(),
                name: value.long.name,
                avatar: "".to_string(), // TODO -> Unsure if this is available // Is a filename from the archive
                email: "".to_string(),  // TODO -> Unsure if this is available
                uuid: value.long.id,
                domain: format!("https://{}/", value.whoami.url),
            }
        }
    }
}

pub mod file {
    use crate::sources::op::one_pux;
    use chrono::{DateTime, FixedOffset};
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Reference {
        pub id: String,

        pub name: String,

        pub size: usize,

        pub content_path: String,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Attrs {
        /// The id of the item for this file.
        pub id: String,

        /// The files name.
        pub title: String,

        /// The times this file has been modified.
        pub version: String,

        /// A reference to the vault which contains this file.
        pub vault: super::vault::Reference,

        /// The file's size in a human readable format.
        #[serde(rename = "overview.ainfo")]
        pub info: String,

        /// The uuid of the user who last edited this file.
        pub last_edited_by: String,

        /// The time this file was created.
        pub created_at: DateTime<FixedOffset>,

        /// The time this file was last modified.
        pub updated_at: DateTime<FixedOffset>,
    }

    impl From<Reference> for one_pux::item::DocumentAttributes {
        fn from(value: Reference) -> Self {
            one_pux::item::DocumentAttributes {
                file_name: value.name,
                document_id: value.id,
                decrypted_size: value.size,
            }
        }
    }
}

// TODO -> Items may be a entire enum type per category with category being the enum variant
pub mod item {
    use super::super::one_pux;
    use crate::config::runtime::RuntimeConfig;
    use crate::sources::op::account::AccountCommon;
    use chrono::{DateTime, FixedOffset};
    use clap::ValueEnum;
    use lib::anyhow::Context;
    use rayon::prelude::*;
    use serde::{Deserialize, Serialize};
    use serde_json::from_slice;
    use std::fmt::{Display, Formatter};
    use std::process::Command;
    use tracing::{instrument, trace};

    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[serde(rename_all = "SCREAMING_SNAKE_CASE", tag = "category")]
    pub enum Item {
        /// Additional is the value of the field `username`
        Login {
            #[serde(flatten)]
            attrs: Attrs,
        },
        CreditCard {
            #[serde(flatten)]
            attrs: Attrs,
        },
        /// Additional is the value of the field `notesPlain`
        SecureNote {
            #[serde(flatten)]
            attrs: Attrs,
        },
        /// Additional seems to be a format of fields like `{firstName} {lastName}`
        Identity {
            #[serde(flatten)]
            attrs: Attrs,
        },
        /// Represents an uploaded file.
        /// Additional is the size of the file in a human readable format.
        Document {
            #[serde(flatten)]
            attrs: Attrs,

            #[serde(default)]
            files: Vec<super::file::Reference>,
        },
        /// Additional is the value of the field `product_version`
        SoftwareLicense {
            #[serde(flatten)]
            attrs: Attrs,
        },
        /// Additional is the value of the field `hostname`
        Database {
            #[serde(flatten)]
            attrs: Attrs,
        },
        /// Additional is the value of the field `number`
        DriverLicense {
            #[serde(flatten)]
            attrs: Attrs,
        },
        /// Additional is the value of the field `TODO`
        OutdoorLicense {
            #[serde(flatten)]
            attrs: Attrs,
        },
        /// Additional is the value of the field `TODO`
        Membership {
            #[serde(flatten)]
            attrs: Attrs,
        },
        /// Additional is the value of the field `TODO`
        Passport {
            #[serde(flatten)]
            attrs: Attrs,
        },
        /// Additional is the value of the field `TODO`
        RewardProgram {
            #[serde(flatten)]
            attrs: Attrs,
        },
        // TODO -> 108
        /// Additional is the value of the field `TODO`
        WirelessRouter {
            #[serde(flatten)]
            attrs: Attrs,
        },
        /// Additional is the value of the field `TODO`
        Server {
            #[serde(flatten)]
            attrs: Attrs,
        },
        /// Additional is the value of the field `pop_username`
        EmailAccount {
            #[serde(flatten)]
            attrs: Attrs,
        },
        /// Additional is the value of the field `type`
        ApiCredential {
            #[serde(flatten)]
            attrs: Attrs,
        },
        /// Additional is the value of the field `date`
        MedicalRecord {
            #[serde(flatten)]
            attrs: Attrs,
        },
        /// Represents an ssh-key.
        /// Additional is the value of the field `public_key`
        SshKey {
            #[serde(flatten)]
            attrs: Attrs,
        },
        /// Seems to be the new standard for creating new item types
        Custom {
            #[serde(flatten)]
            attrs: Attrs,

            /// The unique value which identifies this item type.
            category_id: String,
        },
    }

    #[derive(Default, Debug, Clone, Serialize, Deserialize, ValueEnum)]
    #[serde(rename_all = "SCREAMING_SNAKE_CASE")]
    pub enum State {
        #[default]
        Active,
        Archived,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Attrs {
        /// The unique identifier within the vault.
        id: String,

        /// The user visible and editable title of the item.
        title: String,

        /// If this user has marked this item as a favorite
        #[serde(default)] // TODO -> Only serialize if true
        favorite: bool,

        /// The tags associated with this item, if any.
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        tags: Vec<String>,

        /// The number of times this item has been changed.
        /// Items start at 1.
        version: usize,

        /// The state of the item, either active or archived.
        #[serde(default)]
        state: State,

        /// A reference to the vault which owns this item.
        vault: super::vault::Reference,

        /// The UUID of the User which last edited this item
        last_edited_by: String,

        /// The time at which this item was created.
        created_at: DateTime<FixedOffset>,

        /// The time at which this item was last updated.
        updated_at: DateTime<FixedOffset>,

        // TODO :: Seems to be category specific
        #[serde(default, skip_serializing_if = "Option::is_none")]
        additional_information: Option<String>,

        /// The URLs associated with this item, if any.
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        urls: Vec<super::url::Url>,

        /// The additional sections of this item, if any.
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        sections: Vec<super::section::Section>,

        /// The fields associated with this item, if any.
        #[serde(default, deserialize_with = "super::field::deserialise")]
        fields: Vec<super::field::Field>,
    }

    impl Display for State {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            write!(f, "{self:?}")
        }
    }

    impl From<Item> for one_pux::item::Attrs {
        fn from(val: Item) -> Self {
            let attrs = val.get_attrs().clone();
            one_pux::item::Attrs {
                uuid: attrs.id,
                fav_index: attrs.favorite.into(),
                created_at: attrs.created_at.timestamp(),
                updated_at: attrs.updated_at.timestamp(),
                state: attrs.state.to_string().to_lowercase(),
                category_uuid: val.get_category_id(),
            }
        }
    }

    impl From<Item> for Option<one_pux::item::PasswordDetails> {
        fn from(value: Item) -> Option<one_pux::item::PasswordDetails> {
            let attrs = value.get_attrs();
            let field = attrs
                .fields
                .clone()
                .into_iter()
                .filter_map(|f| match f {
                    super::field::Field::Concealed {
                        password_details, ..
                    } => Some(password_details),
                    _ => None,
                })
                .flatten()
                .next();

            match field {
                None => None,
                Some(details) => Some(one_pux::item::PasswordDetails {
                    password_strength: details.strength.into(),
                    password_base_entropy: details.entropy as f64,
                    password_generated: details.generated,
                }),
            }
        }
    }

    impl From<Item> for one_pux::item::Overview {
        fn from(val: Item) -> Self {
            let attrs = val.get_attrs().clone();
            let watchtower_exclusions: Option<one_pux::item::WatchTowerExclusions> = None; // TODO
            let password_details: Option<one_pux::item::PasswordDetails> = val.into();

            one_pux::item::Overview {
                subtitle: attrs.additional_information.unwrap_or_default(),
                icons: None, // TODO
                urls: attrs.urls.clone().into_iter().map(super::url::Url::into).collect(),
                tags: attrs.tags,
                title: attrs.title,
                url: attrs.urls.into_iter().find(|u| u.primary).map(|u| u.href).unwrap_or_default(),
                password_details,
                watchtower_exclusions,
            }
        }
    }

    impl From<Item> for one_pux::item::Details {
        fn from(value: Item) -> one_pux::item::Details {
            let mut fields = value.get_attrs().fields.clone();

            let mut password_history = None;
            let mut removed = 0usize;
            let login_fields = fields
                .clone()
                .into_iter()
                .enumerate()
                .filter(|(_, f)| f.is_login_field())
                .inspect(|(_, f)| match f {
                    // This is a bit of a hack, but it works for now
                    super::field::Field::Concealed {
                        password_details, ..
                    } => {
                        if password_details.is_some() {
                            let into = password_details.clone().unwrap().into();
                            let _ = password_history.insert(into);
                        }
                    }
                    _ => (),
                })
                .map(|(i, _)| {
                    let field = fields.remove(i - removed);
                    removed += 1;
                    field
                })
                .map(|f| f.into())
                .map(|mut f: one_pux::item::Field| {
                    f.id = "".to_string(); // Login Fields have their Id's empty.
                    f
                })
                .collect::<Vec<one_pux::item::Field>>();

            removed = 0usize;
            let notes_plain = fields
                .clone()
                .into_iter()
                .enumerate()
                .find(|(_, f)| f.is_notes_field())
                .map(|(i, _)| {
                    let field = fields.remove(i - removed);
                    removed += 1;
                    field
                })
                .and_then(|f| f.get_attrs().value.clone());

            // TODO -> This is a bit of a hack, but it works for now
            // TODO -> Sections aren't in the right order, but that's not a big deal
            let mut sections = value
                .get_attrs()
                .sections
                .clone()
                .into_iter()
                .map(|s| s.into())
                .collect::<Vec<one_pux::item::AdditionalSection>>();
            for field in fields {
                let attrs = (&field).get_attrs();

                if attrs.section.as_ref().is_none() {
                    if sections.is_empty()
                        || sections
                            .iter()
                            .find(|s| s.title.is_empty() && s.name.is_empty())
                            .is_none()
                    {
                        sections.insert(
                            0,
                            one_pux::item::AdditionalSection {
                                name: "".to_string(),
                                title: "".to_string(),
                                fields: vec![],
                                hide_add_another_field: false,
                            },
                        )
                    }

                    sections.first_mut().unwrap().fields.push(field.into());
                    continue;
                }

                sections
                    .iter_mut()
                    .find(|s| s.name == attrs.section.as_ref().unwrap().id)
                    .map(|s| s.fields.push(field.into()));
            }

            let document_attributes = match value {
                Item::Document { files, .. } => Some(files.first().map(|f| f.clone().into())),
                _ => None,
            }
            .flatten();

            one_pux::item::Details {
                login_fields,
                notes_plain,
                sections,
                password_history: password_history
                    .context("Unwrap password history")
                    .unwrap_or_default(),
                document_attributes,
            }
        }
    }

    impl From<Item> for one_pux::item::Item {
        // TODO -> Don't clone twice
        fn from(value: Item) -> one_pux::item::Item {
            let attrs = value.clone().into();
            let overview = value.clone().into();
            let details = value.into();

            one_pux::item::Item {
                attrs,
                overview,
                details,
            }
        }
    }

    impl Item {
        pub fn get_category_id(&self) -> String {
            match self {
                Item::Login { .. } => "001",
                Item::CreditCard { .. } => "002",
                Item::SecureNote { .. } => "003",
                Item::Identity { .. } => "004",
                Item::Document { .. } => "006",
                Item::SoftwareLicense { .. } => "100",
                Item::Database { .. } => "102",
                Item::DriverLicense { .. } => "103",
                Item::OutdoorLicense { .. } => "104",
                Item::Membership { .. } => "105",
                Item::Passport { .. } => "106",
                Item::RewardProgram { .. } => "107",
                Item::WirelessRouter { .. } => "109",
                Item::Server { .. } => "110",
                Item::EmailAccount { .. } => "111",
                Item::ApiCredential { .. } => "112",
                Item::MedicalRecord { .. } => "113",
                Item::SshKey { .. } => "114",
                Item::Custom { .. } => "115",
            }
            .to_string()
        }

        pub fn get_attrs(&self) -> &Attrs {
            match self {
                Item::Login { attrs, .. } => attrs,
                Item::CreditCard { attrs, .. } => attrs,
                Item::SecureNote { attrs, .. } => attrs,
                Item::Identity { attrs, .. } => attrs,
                Item::Document { attrs, .. } => attrs,
                Item::SoftwareLicense { attrs, .. } => attrs,
                Item::Database { attrs, .. } => attrs,
                Item::DriverLicense { attrs, .. } => attrs,
                Item::OutdoorLicense { attrs, .. } => attrs,
                Item::Membership { attrs, .. } => attrs,
                Item::Passport { attrs, .. } => attrs,
                Item::RewardProgram { attrs, .. } => attrs,
                Item::WirelessRouter { attrs, .. } => attrs,
                Item::Server { attrs, .. } => attrs,
                Item::EmailAccount { attrs, .. } => attrs,
                Item::ApiCredential { attrs, .. } => attrs,
                Item::MedicalRecord { attrs, .. } => attrs,
                Item::SshKey { attrs, .. } => attrs,
                Item::Custom { attrs, .. } => attrs,
            }
        }

        fn raw(vault_id: &String, mut command: Command) -> Vec<u8> {
            command
                .args(["item", "list"])
                .args(["--vault", vault_id, "--format=json"])
                .output()
                .context("Unwrap vault list")
                .unwrap()
                .stdout
        }

        fn raw_long(vault_id: &String, item_id: &String, mut command: Command) -> Vec<u8> {
            command
                .args(["item", "get", item_id])
                .args(["--vault", vault_id, "--format=json"])
                .output()
                .context("Unwrap vault item")
                .unwrap()
                .stdout
        }

        #[instrument]
        pub fn parse(
            vault: super::vault::Vault,
            account: &&dyn AccountCommon,
            config: &RuntimeConfig,
        ) -> Vec<Item> {
            trace!("Requesting Items from {vault}");

            let raw = Self::raw(&vault.reference.id, account.command(config));
            let parsed = from_slice::<Vec<Item>>(&raw).context("Deserialize items list").unwrap();
            parsed
                .into_par_iter()
                .map(|item| {
                    Self::raw_long(
                        &vault.reference.id,
                        &item.get_attrs().id,
                        account.command(config),
                    )
                })
                .map(|raw| from_slice::<Item>(&raw).context("Deserialize item").unwrap())
                .collect() // TODO :: Sort by creation date?
        }
    }
}

pub mod vault {
    use crate::config::runtime::RuntimeConfig;
    use crate::sources::op::account::AccountCommon;
    use crate::sources::op::cli::CliGetter;
    use crate::sources::op::one_pux;
    use async_trait::async_trait;
    use chrono::{DateTime, FixedOffset};
    use lib::anyhow::Context;
    use serde::{Deserialize, Serialize};
    use serde_json::from_slice;
    use std::fmt::Display;
    use std::process::Command;
    use tracing::trace;

    // TODO -> Possible to merge cli & 1pux types?
    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[serde(rename_all = "SCREAMING_SNAKE_CASE")]
    pub enum Type {
        Personal,
        Shared, // ?? // TODO
        UserCreated,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[serde(rename = "vault")]
    pub struct Reference {
        pub id: String,
        pub name: String,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Vault {
        #[serde(flatten)]
        pub reference: Reference,
        pub content_version: usize,
        pub attribute_version: usize,
        pub items: usize,
        #[serde(rename = "type")]
        pub vault_type: Type,
        pub created_at: DateTime<FixedOffset>,
        pub updated_at: DateTime<FixedOffset>,
    }

    impl Display for Reference {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}-{}", self.name, self.id)
        }
    }

    impl Display for Vault {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            self.reference.fmt(f)
        }
    }

    #[async_trait]
    impl CliGetter<Vec<Reference>> for Reference {
        async fn get(
            config: &RuntimeConfig,
            envs: &[(&str, &str)],
            args: &[&str],
        ) -> lib::anyhow::Result<Vec<Reference>>
        where
            Self: Sized,
        {
            let mut command = Self::prepared(config, envs, args);
            command.args(["vault", "list", "--format=json"]);
            trace!("Running command {:?}", command);

            let output = command.output()?;
            let stdout = output.stdout;
            let stderr = output.stderr;

            if !output.status.success() {
                return Err(lib::anyhow::anyhow!(
                    "Command {:?} failed with status {:?} and stderr {:?}",
                    command,
                    output.status,
                    String::from_utf8_lossy(&stderr)
                ));
            }

            let parsed = from_slice::<Vec<Reference>>(&stdout)?;
            Ok(parsed)
        }
    }

    impl Vault {
        fn raw(vault_id: &String, mut command: Command) -> Vec<u8> {
            command
                .args(["vault", "get", vault_id, "--format=json"])
                .output()
                .context("Unwrap vault get")
                .unwrap()
                .stdout
        }

        pub fn parse(account: &&dyn AccountCommon, config: &RuntimeConfig) -> Vec<Vault> {
            account
                .vaults()
                .into_iter()
                .inspect(|vault| trace!("Requesting Vault {vault}",))
                .map(|reference| Self::raw(&reference.id, account.command(config)))
                .inspect(|output| trace!("Parsing Vault JSON {}", String::from_utf8_lossy(output)))
                .map(|output| {
                    from_slice::<Vault>(output.as_slice()).context("Deserialise vault").unwrap()
                })
                .collect()
        }
    }

    impl From<Type> for one_pux::vault::Type {
        fn from(val: Type) -> Self {
            match val {
                Type::Personal => one_pux::vault::Type::P,
                Type::Shared => one_pux::vault::Type::E,
                Type::UserCreated => one_pux::vault::Type::U,
            }
        }
    }

    impl From<Vault> for one_pux::vault::Attrs {
        fn from(val: Vault) -> Self {
            one_pux::vault::Attrs {
                uuid: val.reference.id,
                desc: "".to_string(),   // TODO
                avatar: "".to_string(), // TODO
                name: val.reference.name,
                vault_type: val.vault_type.into(),
            }
        }
    }
}

pub mod url {
    use super::super::one_pux;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Url {
        #[serde(default)]
        pub label: Option<String>,
        #[serde(default)]
        pub primary: bool,
        pub href: String,
    }

    impl Default for Url {
        fn default() -> Self {
            Url {
                label: None,
                primary: false,
                href: "".to_string(),
            }
        }
    }

    impl From<Url> for one_pux::item::UrlObject {
        fn from(val: Url) -> Self {
            one_pux::item::UrlObject {
                url: val.href,
                label: val.label.unwrap_or_default(),
                mode: "default".to_string(), // Unable to get from CLI
            }
        }
    }
}

pub mod field {
    use super::super::one_pux;
    use serde::de::{SeqAccess, Visitor};
    use serde::{Deserialize, Deserializer, Serialize};
    use std::fmt;
    use std::str::FromStr;
    use tracing::debug;

    #[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
    #[serde(rename_all = "UPPERCASE")]
    pub enum Purpose {
        Username,
        Password,
        Notes,
    }

    #[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
    #[serde(rename_all = "UPPERCASE")]
    pub enum PasswordStrength {
        Terrible,
        Good,
    }

    #[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
    pub struct PasswordDetails {
        // TODO -> Might be deprecated due to root root level entropy field
        #[serde(default)]
        pub entropy: usize,
        #[serde(default)]
        pub generated: bool, // TODO -> Serialise only if true
        pub strength: PasswordStrength,
        #[serde(default)]
        pub history: Vec<String>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
    pub struct Attrs {
        // TODO :: Seems like its an enum with the rename_all of camelCase
        /// An internal identifier for the field, not shown to user.
        pub id: String,

        /// If present references the additional section the field belongs to.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub section: Option<super::section::Section>,

        // TODO -> docs
        #[serde(default)]
        pub purpose: Option<Purpose>,

        /// The user visible and editable label for the field.
        pub label: String,

        /// The value/data of the field if present.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub value: Option<String>,

        /// Points to the fields location in the vault
        /// ## Format: `op://{vault}/{field type}/({section}/|empty){label}`
        pub reference: String,
    }

    #[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
    #[serde(rename_all = "SCREAMING_SNAKE_CASE", tag = "type")]
    pub enum Field {
        /// Represents a string field, such as a username or notes.
        String {
            #[serde(flatten)]
            attrs: Attrs,
        },
        /// Represents a concealed field, such as a password.
        Concealed {
            #[serde(flatten)]
            attrs: Attrs,
            #[serde(default, skip_serializing_if = "Option::is_none")]
            password_details: Option<PasswordDetails>,
        },
        Otp {
            #[serde(flatten)]
            attrs: Attrs,
        },
        CreditCardNumber {
            #[serde(flatten)]
            attrs: Attrs,
        },
        /// Shows a dropdown menu of pre-defined values for the field based of `attrs.id`
        Menu {
            #[serde(flatten)]
            attrs: Attrs,
        },
        /// Shows a calendar widget for the field.
        /// Value is formatted as a unix timestamp at the time 12:01pm on the selected date.
        Date {
            #[serde(flatten)]
            attrs: Attrs,
        },
        /// Allows the input of a month and year only.
        /// Value is formatted as `YYYYMM`.
        MonthYear {
            #[serde(flatten)]
            attrs: Attrs,
        },
        Url {
            #[serde(flatten)]
            attrs: Attrs,
        },
        Phone {
            #[serde(flatten)]
            attrs: Attrs,
        },
        /// Value seems to be formatted as `street, suburb, state, country, zip`
        Address {
            #[serde(flatten)]
            attrs: Attrs,
        },
        Email {
            #[serde(flatten)]
            attrs: Attrs,
        },
        #[serde(rename = "SSHKEY")]
        SshKey {
            #[serde(flatten)]
            attrs: Attrs,
        },
    }

    impl Field {
        const LOGIN_PURPOSES: [Purpose; 2] = [Purpose::Username, Purpose::Password];

        pub fn is_login_field(&self) -> bool {
            let attrs = match self {
                Field::String { attrs, .. } => attrs,
                Field::Concealed { attrs, .. } => attrs,
                _ => return false,
            };

            attrs.section.is_none()
                && attrs.purpose.as_ref().is_some_and(|p| Self::LOGIN_PURPOSES.contains(&p))
        }

        pub fn is_notes_field(&self) -> bool {
            match self {
                Field::String { attrs, .. } => {
                    attrs.section.is_none()
                        && attrs.purpose.as_ref().is_some_and(|p| p == &Purpose::Notes)
                }
                _ => false,
            }
        }

        pub fn get_attrs(&self) -> &Attrs {
            match self {
                Field::String { attrs, .. } => attrs,
                Field::Concealed { attrs, .. } => attrs,
                Field::Otp { attrs, .. } => attrs,
                Field::CreditCardNumber { attrs, .. } => attrs,
                Field::Menu { attrs, .. } => attrs,
                Field::Date { attrs, .. } => attrs,
                Field::MonthYear { attrs, .. } => attrs,
                Field::Url { attrs, .. } => attrs,
                Field::Phone { attrs, .. } => attrs,
                Field::Address { attrs, .. } => attrs,
                Field::Email { attrs, .. } => attrs,
                Field::SshKey { attrs, .. } => attrs,
            }
        }
    }

    impl From<PasswordStrength> for usize {
        fn from(val: PasswordStrength) -> Self {
            match val {
                PasswordStrength::Terrible => 20,
                PasswordStrength::Good => 40,
            }
        }
    }

    impl From<Field> for one_pux::item::FieldType {
        fn from(val: Field) -> Self {
            match val {
                Field::String { .. } => one_pux::item::FieldType::Text,
                Field::Concealed { .. } => one_pux::item::FieldType::Password,
                field => panic!("Unsupported field type: {:?}", field),
            }
        }
    }

    impl From<Purpose> for one_pux::item::FieldDesignation {
        fn from(val: Purpose) -> Self {
            match val {
                Purpose::Username => one_pux::item::FieldDesignation::Username,
                Purpose::Password => one_pux::item::FieldDesignation::Password,
                Purpose::Notes => one_pux::item::FieldDesignation::None,
            }
        }
    }

    impl From<Field> for one_pux::item::Field {
        fn from(val: Field) -> Self {
            let attrs = val.get_attrs().clone();
            one_pux::item::Field {
                id: match &attrs.label {
                    l if l == &attrs.label => "",
                    l => l,
                }
                .to_string(), // TODO :: Clear if same as name i think, needs checking
                name: attrs.label,
                value: attrs.value.unwrap_or_default(),
                designation: attrs.purpose.map(|p| p.into()).unwrap_or_default(),
                field_type: val.into(),
            }
        }
    }

    impl From<PasswordDetails> for Vec<one_pux::item::PasswordHistory> {
        fn from(val: PasswordDetails) -> Self {
            val.history
                .iter()
                .map(|h| one_pux::item::PasswordHistory {
                    value: h.clone(),
                    time: 0, // TODO -> I'm unsure if the cli can expose this
                })
                .collect::<Vec<one_pux::item::PasswordHistory>>()
        }
    }

    // TODO :: SSHKey support
    impl From<Field> for one_pux::section::Field {
        fn from(val: Field) -> Self {
            let multiline = val.get_attrs().value.clone().is_some_and(|v| v.contains('\n'));

            match val {
                Field::String { attrs } => one_pux::section::Field {
                    title: attrs.label,
                    id: attrs.id,
                    value: one_pux::section::Value::String(attrs.value.unwrap_or_default()),
                    multiline,
                    ..Default::default()
                },
                Field::Concealed { attrs, .. } => one_pux::section::Field {
                    title: attrs.label,
                    id: attrs.id,
                    value: one_pux::section::Value::Concealed(attrs.value.unwrap_or_default()),
                    multiline,
                    ..Default::default()
                },
                Field::Otp { attrs } => one_pux::section::Field {
                    title: attrs.label,
                    id: attrs.id,
                    value: one_pux::section::Value::TOTP(attrs.value.unwrap_or_default()),
                    multiline,
                    input_traits: one_pux::section::InputTraits {
                        correction: one_pux::section::Correction::No,
                        capitalization: one_pux::section::Capitalization::None,
                        ..Default::default()
                    },
                    ..Default::default()
                },
                Field::CreditCardNumber { attrs } => one_pux::section::Field {
                    title: attrs.label,
                    id: attrs.id,
                    value: one_pux::section::Value::CreditCardNumber(
                        attrs.value.unwrap_or_default(),
                    ),
                    guarded: true,
                    multiline,
                    clipboard_filter: Some("0123456789".to_string()),
                    input_traits: one_pux::section::InputTraits {
                        keyboard: one_pux::section::Keyboard::NumberPad,
                        ..Default::default()
                    },
                    ..Default::default()
                },
                Field::Menu { attrs } => one_pux::section::Field {
                    title: attrs.label,
                    id: attrs.id,
                    value: one_pux::section::Value::Menu(attrs.value.unwrap_or_default()),
                    multiline,
                    ..Default::default()
                },
                Field::Date { attrs } => one_pux::section::Field {
                    title: attrs.label,
                    id: attrs.id,
                    value: one_pux::section::Value::Date(
                        attrs.value.map(|v| usize::from_str(&v).unwrap()),
                    ),
                    multiline,
                    ..Default::default()
                },
                Field::MonthYear { attrs } => one_pux::section::Field {
                    title: attrs.label,
                    id: attrs.id,
                    value: one_pux::section::Value::MonthYear(
                        attrs.value.map(|v| usize::from_str(&v).unwrap()),
                    ),
                    multiline,
                    ..Default::default()
                },
                Field::Url { attrs } => one_pux::section::Field {
                    title: attrs.label,
                    id: attrs.id,
                    value: one_pux::section::Value::Url(attrs.value.unwrap_or_default()),
                    multiline,
                    input_traits: one_pux::section::InputTraits {
                        keyboard: one_pux::section::Keyboard::URL,
                        ..Default::default()
                    },
                    ..Default::default()
                },
                Field::Phone { attrs } => one_pux::section::Field {
                    title: attrs.label,
                    id: attrs.id,
                    value: one_pux::section::Value::Phone(attrs.value.unwrap_or_default()),
                    multiline,
                    input_traits: one_pux::section::InputTraits {
                        keyboard: one_pux::section::Keyboard::NamePhonePad,
                        ..Default::default()
                    },
                    ..Default::default()
                },
                Field::Address { attrs } => {
                    let value = attrs.value.unwrap();
                    let mut split = value.split(", ").collect::<Vec<&str>>();
                    let mut next = || -> String {
                        let value = split.remove(0);
                        if value == "<nil>" {
                            return String::new();
                        }

                        value.to_string()
                    };

                    one_pux::section::Field {
                        title: attrs.label,
                        id: attrs.id,
                        value: one_pux::section::Value::Address {
                            street: next(),
                            city: next(),
                            state: next(),
                            zip: next(),
                            country: next(),
                        },
                        multiline,
                        ..Default::default()
                    }
                }
                Field::Email { attrs } => one_pux::section::Field {
                    title: attrs.label,
                    id: attrs.id,
                    value: one_pux::section::Value::Email {
                        email_address: attrs.value.unwrap_or_default(),
                        provider: None, // TODO
                    },
                    guarded: true,
                    multiline,
                    input_traits: one_pux::section::InputTraits {
                        keyboard: one_pux::section::Keyboard::EmailAddress,
                        ..Default::default()
                    },
                    ..Default::default()
                },
                Field::SshKey { attrs } => one_pux::section::Field {
                    title: attrs.label,
                    id: attrs.id,
                    value: one_pux::section::Value::SshKey {
                        private_key: attrs.value.clone().unwrap_or_default(),
                        metadata: one_pux::section::SshKeyMetadata {
                            private_key: attrs.value.unwrap_or_default(),
                            // These values are all their own individual fields
                            public_key: "".to_string(),  // TODO
                            fingerprint: "".to_string(), // TODO
                            key_type: "".to_string(),    // TODO
                        },
                    },
                    guarded: true,
                    multiline,
                    ..Default::default()
                },
            }
        }
    }

    pub(super) fn deserialise<'de, D>(deserializer: D) -> Result<Vec<Field>, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct FieldVisitor;

        impl<'de> Visitor<'de> for FieldVisitor {
            type Value = Vec<Field>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("vec of Fields")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                debug!(
                    "Visiting seq with hint of {}",
                    seq.size_hint().unwrap_or(usize::MAX)
                );
                let mut fields = vec![];

                while let Some(field) = seq.next_element::<Field>()? {
                    debug!("Visiting field: {:?}", field);
                    fields.push(field);
                }

                Ok(fields)
            }
        }

        deserializer.deserialize_any(FieldVisitor)
    }
}

pub mod section {
    use super::super::one_pux;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
    pub struct Section {
        pub id: String,
        pub label: Option<String>,
    }

    impl Section {
        pub fn into(self) -> one_pux::item::AdditionalSection {
            one_pux::item::AdditionalSection {
                title: self.label.unwrap_or_default(),
                name: self.id.clone(),
                fields: vec![],
                hide_add_another_field: false,
            }
        }
    }
}
