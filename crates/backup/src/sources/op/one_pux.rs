/// Zip format
/// export.data is the main archive file
/// export.attributes is the metadata for the archive
/// files.* are the files in the archive
/// Kind of shitty but still relevant info at https://support.1password.com/1pux-format/

/// The 1PUX version which is targeted for compatibility.
pub const ONE_PUX_VERSION: u8 = 3;

pub mod export {
    use crate::config::runtime::RuntimeConfig;
    use crate::sources::op::cli;
    use chrono::Local;
    use lib::anyhow::Result;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Data {
        pub accounts: Vec<super::account::Account>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Export {
        pub data: Data,
        pub attributes: super::attributes::Attributes,
        pub files: Vec<super::file::File>,
        pub name: String,
    }

    impl Export {
        pub fn from(
            value: &dyn super::super::account::AccountCommon,
            config: &RuntimeConfig,
        ) -> Result<Self> {
            let pairs = cli::vault::Vault::parse(&value, config)
                .into_iter()
                .map(|vault| (vault.clone(), cli::item::Item::parse(vault, &value, config)))
                .collect::<Vec<(cli::vault::Vault, Vec<cli::item::Item>)>>();

            let data = vec![super::account::Account {
                attrs: value.account().clone().into(),
                vaults: pairs
                    .into_iter()
                    .map(|(vault, items)| super::vault::Vault {
                        attrs: vault.into(),
                        items: items.into_iter().map(|item| item.into()).collect(),
                    })
                    .collect(),
            }];

            let name = format!(
                "1PasswordExport-{uuid}-{time}.1pux",
                uuid = value.account().long.id,
                time = Local::now().format("%Y%m%d-%H%M%S")
            );

            Ok(Export {
                name,
                data: Data { accounts: data },
                attributes: super::attributes::Attributes::default(),
                files: vec![], // TODO
            })
        }
    }
}

pub mod file {
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct File {
        pub name: String,
        pub data: Vec<u8>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Detail {
        /// The name of the file which is stored within the archive
        /// This name is not a path and is only the name of the file.
        /// You will find these files at `/files/{file_id}`.
        pub file_id: String,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Icon {
        pub detail: Detail,
    }
}

pub mod attributes {
    use serde::{Deserialize, Serialize};

    /// An additional file which gets packaged alongside the export,
    /// This file contains metadata about the export which can be used to,
    /// determine how to handle deserialization.
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Attributes {
        /// The 1PUX format version
        pub version: u8,
        /// The description of the export
        pub description: String,
        /// The unix epoch timestamp of when the export was created
        pub timestamp: i64,
    }

    impl Default for Attributes {
        fn default() -> Self {
            Self {
                version: super::ONE_PUX_VERSION,
                description: String::from("1Password Unencrypted Export"),
                timestamp: chrono::Utc::now().timestamp(),
            }
        }
    }
}

pub mod account {
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Account {
        pub attrs: Attrs,
        pub vaults: Vec<super::vault::Vault>,
    }

    // Might be missing accountName
    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Attrs {
        pub account_name: String, // TODO :: Org name or just account name i think.
        pub name: String,
        pub avatar: String, // TODO :: References a file in the zip archive
        pub email: String,
        pub uuid: String,
        pub domain: String,
    }
}

pub mod vault {
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Vault {
        pub attrs: Attrs,
        pub items: Vec<super::item::Item>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum Type {
        P,
        E,
        U,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Attrs {
        pub uuid: String,
        #[serde(default)]
        pub desc: String, // TODO
        #[serde(default)]
        pub avatar: String, // TODO -> References a file in the zip archive
        pub name: String,
        #[serde(rename = "type")]
        pub vault_type: Type,
    }
}

pub mod section {
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Attrs {
        pub title: String,
        #[serde(default)]
        pub name: Option<String>,
        pub fields: Vec<Field>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Field {
        pub title: String,
        pub id: String,
        pub value: Value,
        pub guarded: bool, // ?
        pub multiline: bool,
        pub dont_generate: bool,
        pub input_traits: InputTraits,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct SsoItem {
        pub vault_uuid: String,
        pub vault_name: String,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct SshKeyMetadata {
        pub private_key: String,
        pub public_key: String,
        pub fingerprint: String,
        pub key_type: String,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub enum Value {
        String(String),
        TOTP(String),
        Concealed(String),
        Menu(String),
        Phone(String),
        MonthYear(String),
        CreditCardNumber(String),
        Email {
            email_address: String,
            #[serde(default)]
            provider: Option<String>,
        },
        SsoLogin {
            provider: String,
            item: SsoItem,
        },
        SshKey {
            private_key: String,
            metadata: SshKeyMetadata,
        },
    }

    #[derive(Default, Debug, Clone, Serialize, Deserialize)]
    pub struct InputTraits {
        keyboard: Keyboard,
        correction: Correction,
        capitalization: Capitalization,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[serde(rename_all = "lowercase")]
    #[derive(Default)]
    pub enum Correction {
        #[default]
        Default,
        No,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    #[derive(Default)]
    pub enum Keyboard {
        #[default]
        Default,
        NumbersAndPunctuation,
        NumberPad,
        NamePhonePad,
        EmailAddress,
        URL,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    #[derive(Default)]
    pub enum Capitalization {
        #[default]
        Default,
        Words,
        None,
        Sentences,
        AllCharacters,
    }
}

pub mod item {
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Item {
        #[serde(flatten)]
        pub attrs: Attrs,
        pub details: Details,
        pub overview: Overview,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Attrs {
        pub uuid: String,
        /// Actually represented by either a 1 or 0
        /// * 1 = Favorite
        /// * 0 = Not Favorite
        pub fav_index: i32,
        /// Unix timestamp in seconds
        pub created_at: i64,
        /// Unix timestamp in seconds
        pub updated_at: i64,
        pub state: String, // enum? // Only present if archived, default to active
        pub category_uuid: String,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum FieldType {
        #[serde(rename = "T")]
        Text,
        #[serde(rename = "E")]
        Email,
        #[serde(rename = "U")]
        URL,
        #[serde(rename = "N")]
        Number,
        #[serde(rename = "P")]
        Password,
        #[serde(rename = "A")]
        TextArea,
        #[serde(rename = "TEL")]
        Telephone,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub enum FieldDesignation {
        Username,
        Password,
        None,
    }

    impl FieldDesignation {
        pub fn is_none(&self) -> bool {
            matches!(self, FieldDesignation::None)
        }
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct AdditionalSection {
        pub title: String,
        pub name: String,
        pub fields: Vec<super::section::Field>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct PasswordHistory {
        pub value: String,
        pub time: i64,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Field {
        pub value: String,
        pub id: String,
        pub name: String,
        pub field_type: FieldType,
        #[serde(skip_serializing_if = "FieldDesignation::is_none")]
        pub designation: FieldDesignation,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct UrlObject {
        pub label: String,
        pub url: String,
        pub mode: String, // TODO :: Enum
    }

    /// TODO -> I Don't think this is something i can get from the CLI
    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct WatchTowerExclusions {
        pub compromised: bool,
        pub vulnerable: bool,
        pub reused: bool,
        pub weak: bool,
        pub unsecured: bool,
        pub inactive_mfa: bool,
        pub expiring: bool,
        pub lastpass: bool,
        pub wrong_account: bool,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct PasswordDetails {
        /// A password strength score for the item between 0 and 100.
        #[serde(default, rename = "ps")]
        pub password_strength: usize,
        #[serde(default, rename = "pbe")]
        pub password_base_entropy: f64,
        /// If this item was generated by the 1Password password generator.
        #[serde(default, rename = "pgrng")]
        pub password_generated: bool,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Overview {
        /// ??
        pub subtitle: String,
        /// The user defined icon for the item.
        #[serde(default)]
        pub icons: Option<super::file::Icon>, // TODO -> Can't seem to get from CLI
        /// A list of URLs related to this item.
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        pub urls: Vec<UrlObject>,
        /// The user defined tags for the item.
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        pub tags: Vec<String>,
        /// The title of the item.
        pub title: String,
        /// The URL of the primary url associated with this item.
        pub url: String,

        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub password_details: Option<PasswordDetails>,

        #[serde(default)]
        pub watchtower_exclusions: Option<WatchTowerExclusions>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct DocumentAttributes {
        pub file_name: String,
        pub document_id: String,
        pub decrypted_size: usize,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Details {
        pub login_fields: Vec<Field>,

        /// The notes for the item.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub notes_plain: Option<String>,

        pub sections: Vec<AdditionalSection>,

        pub password_history: Vec<PasswordHistory>,

        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub document_attributes: Option<DocumentAttributes>,
    }
}
