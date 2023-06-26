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
    use indicatif::{MultiProgress, ProgressBar};
    use lib::anyhow::{anyhow, Result};
    use serde::{Deserialize, Serialize};
    use tracing::{error, warn};
    use lib::anyhow;
    use crate::sources::getter::CommandFiller;

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
        // TODO -> Better error handling
        pub async fn from<A: super::super::account::AccountCommon + CommandFiller>(
            value: &A,
            config: &RuntimeConfig,
            bars: (&ProgressBar, &MultiProgress),
        ) -> Result<(Self, Vec<anyhow::Error>)> {
            let vaults = cli::vault::Vault::parse(value, config).await?;
            if vaults.is_empty() {
                return Err(anyhow!("No vaults found in account {}", value));
            }

            let mut errors = vec![];
            let mut finished = vec![];
            for vault in vaults {
                let attrs = vault.clone().into();

                let items = match cli::item::Item::parse(vault.clone(), value, config, bars) {
                    Ok(items) => items,
                    Err(e) => {
                        error!("Failed to parse items for vault {vault}: {e}");
                        errors.push(e);
                        continue;
                    }
                };

                if items.is_empty() {
                    warn!("No items found in vault {vault}");
                }

                let mut parsed = vec![];
                for item in items {
                    let item: super::item::Item = match item.try_into() {
                        Ok(item) => item,
                        Err(e) => {
                            error!("Failed to parse item : {e}");
                            errors.push(e);
                            continue;
                        }
                    };

                    parsed.push(item);
                }

                finished.push(super::vault::Vault {
                    attrs,
                    items: parsed,
                })
            }

            let data = vec![super::account::Account {
                attrs: value.account().clone().into(),
                vaults: finished,
            }];

            let name = format!(
                "1PasswordExport-{uuid}-{time}.1pux",
                uuid = value.account().get_attrs().identifier.id(),
                time = Local::now().format("%Y%m%d-%H%M%S")
            );

            let export = Export {
                name,
                data: Data { accounts: data },
                attributes: super::attributes::Attributes::default(),
                files: vec![], // TODO
            };

            Ok((export, errors))
        }
    }
}

pub mod identifier {
    use serde::{Deserialize, Serialize};

    #[derive(Default, Debug, Clone, Serialize, Deserialize)]
    pub struct Identifier {
        pub id: String,
        pub title: String,
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

    // TODO :: Merge with loginField?
    #[derive(Default, Debug, Clone, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Field {
        /// The user-facing title of the field
        pub title: String,

        /// The internal identifier of the field
        pub id: String,

        /// The value of the field
        #[serde(skip_serializing_if = "Value::should_skip_serializing")]
        pub value: Value,

        // TODO -> I think this has to do with default fields of a type, so fields that can't be deleted but can be left empty?
        pub guarded: bool,

        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub clipboard_filter: Option<String>,

        // TODO -> Seems to be a set based on the id of the field
        pub multiline: bool,

        // TODO
        pub dont_generate: bool,

        // TODO -> Only place i've found it is in `medical record` fields
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub placeholder: Option<String>,

        // TODO
        pub input_traits: InputTraits,
    }

    #[derive(Default, Debug, Clone, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct SsoItem {
        pub vault_uuid: String,
        pub vault_name: String,
    }

    #[derive(Default, Debug, Clone, Serialize, Deserialize)]
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
        Concealed(String),
        #[serde(rename = "totp")]
        TOTP(String),
        Menu(String),
        Phone(String),
        Url(String),
        MonthYear(Option<usize>),
        Date(Option<usize>),
        CreditCardNumber(String),
        Reference(String),
        Address {
            street: String,
            city: String,
            country: String,
            zip: String,
            state: String,
        },
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
            metadata: SshKeyMetadata, // TODO :: From other fields
        },
    }

    // TODO :: Seems to be based off the id of the field, eg `name` -> `capitalization == words`
    #[derive(Default, Debug, Clone, Serialize, Deserialize)]
    pub struct InputTraits {
        pub keyboard: Keyboard,
        pub correction: Correction,
        pub capitalization: Capitalization,
    }

    #[derive(Default, Debug, Clone, Serialize, Deserialize)]
    #[serde(rename_all = "lowercase")]
    pub enum Correction {
        #[default]
        Default,
        No,
    }

    #[derive(Default, Debug, Clone, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub enum Keyboard {
        #[default]
        Default,
        NumbersAndPunctuation,
        NumberPad,
        NamePhonePad,
        EmailAddress,
        URL,
    }

    #[derive(Default, Debug, Clone, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub enum Capitalization {
        #[default]
        Default,
        Words,
        None,
        Sentences,
        AllCharacters,
    }

    impl Default for Value {
        fn default() -> Self {
            Value::String(String::new())
        }
    }

    impl Value {
        // TODO :: Use to hide sshkey fields
        fn should_skip_serializing(&self) -> bool {
            match self {
                // Value::Date(None) | Value::MonthYear(None) => true,
                _ => false,
            }
        }
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
        pub state: String, // TODO :: enum? // Only present if archived, default to active
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

    #[derive(Default, Debug, Clone, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub enum FieldDesignation {
        #[default]
        None,
        Username,
        Password,
    }

    impl FieldDesignation {
        pub fn is_none(&self) -> bool {
            matches!(self, FieldDesignation::None)
        }
    }

    #[derive(Default, Debug, Clone, Serialize, Deserialize)]
    pub struct AdditionalSection {
        pub title: String,

        #[serde(default, skip_serializing_if = "String::is_empty")]
        pub name: String,

        pub fields: Vec<super::section::Field>,

        #[serde(default, skip_serializing_if = "super::is_default")]
        pub hide_add_another_field: bool,
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

        #[serde(flatten, default, skip_serializing_if = "Option::is_none")]
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

pub(crate) fn is_default<T: Default + PartialEq>(t: &T) -> bool {
    t == &T::default()
}

pub(crate) fn not_default<T: Default + PartialEq>(t: &T) -> bool {
    t != &T::default()
}
