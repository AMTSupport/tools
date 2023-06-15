use serde::{Deserialize, Serialize};
use super::cli;
use crate::config::runtime::RuntimeConfig;
use crate::sources::op::account::AccountCommon;
use lib::anyhow::Result;
use lib::simplelog::debug;

/// Zip format
/// export.data is the main archive file
/// export.attributes is the metadata for the archive
/// files.* are the files in the archive
/// Kind of shitty but still relavent info at https://support.1password.com/1pux-format/

pub async fn create_export(account: Box<&dyn AccountCommon>, config: &RuntimeConfig) -> Result<Export> {
    let pairs = cli::vault::Vault::parse(&account, &config)
        .into_iter()
        .map(|vault| (vault.clone(), cli::item::Item::parse(vault, &account, &config)))
        .collect::<Vec<(cli::vault::Vault, Vec<cli::item::Item>)>>();

    debug!("pairs: {:#?}", pairs);

    Ok(Export {
        accounts: vec![account::Account {
            attrs: account.account().clone().into(),
            vaults: pairs
                .into_iter()
                .map(|(vault, items)| vault::Vault {
                    attrs: vault.into(),
                    items: items.into_iter().map(|item| item.into()).collect(),
                })
                .collect(),
        }],
    })
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Export {
    pub accounts: Vec<account::Account>,
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
        pub avatar: String,
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
        pub avatar: String, // TODO
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
    pub enum Correction {
        Default,
        No,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub enum Keyboard {
        Default,
        NumbersAndPunctuation,
        NumberPad,
        NamePhonePad,
        EmailAddress,
        URL,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub enum Capitalization {
        Default,
        Words,
        None,
        Sentences,
        AllCharacters,
    }

    impl Default for Correction {
        fn default() -> Self {
            Correction::Default
        }
    }

    impl Default for Keyboard {
        fn default() -> Self {
            Keyboard::Default
        }
    }

    impl Default for Capitalization {
        fn default() -> Self {
            Capitalization::Default
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
            match self {
                FieldDesignation::None => true,
                _ => false,
            }
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
    pub struct Field {
        pub value: String,
        pub id: String,
        pub name: String,
        #[serde(rename = "type")]
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

    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Overview {
        pub subtitle: String,
        #[serde(default)]
        pub urls: Vec<UrlObject>,
        pub title: String,
        pub url: String, // TODO :: Primary URL from urls array
        #[serde(skip_serializing_if = "Option::is_none")]
        pub tags: Option<Vec<String>>, // TODO Skip if none

        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub icons: Option<String>, // ? // Can't seem to get from CLI
        #[serde(default)]
        pub ps: i64, // ?
        #[serde(default)]
        pub pbe: f64, // ?
        #[serde(default)]
        pub pgrng: bool, // ??
        #[serde(default)]
        pub watchtower_exclusions: Option<bool>, // ??
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Details {
        pub login_fields: Vec<Field>,
        pub notes_plain: String,
        pub sections: Vec<AdditionalSection>,
        pub password_history: Vec<PasswordHistory>,
    }
}
