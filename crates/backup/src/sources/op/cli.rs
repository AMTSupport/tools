pub mod account {
    use crate::sources::op::one_pux;
    use chrono::{DateTime, FixedOffset};
    use serde::{Deserialize, Serialize};

    // TODO :: Add more types
    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[serde(rename_all = "SCREAMING_SNAKE_CASE")]
    pub enum Type {
        Member,
        ServiceAccount,
    }

    // TODO :: Add more states
    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[serde(rename_all = "SCREAMING_SNAKE_CASE")]
    pub enum State {
        Active,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Account {
        pub id: String,
        pub name: String,
        pub email: String,
        #[serde(rename = "type")]
        pub account_type: Type,
        #[serde(default)]
        #[serde(skip)]
        pub state: State,
        #[serde(skip)]
        pub created_at: DateTime<FixedOffset>,
        #[serde(skip)]
        pub updated_at: DateTime<FixedOffset>,
        #[serde(skip)]
        pub last_auth_at: DateTime<FixedOffset>,
    }

    impl Default for State {
        fn default() -> Self {
            State::Active
        }
    }

    impl Into<one_pux::account::Attrs> for Account {
        fn into(self) -> one_pux::account::Attrs {
            one_pux::account::Attrs {
                account_name: self.name.clone(), // TODO :: Org name or just account name if personal account
                name: self.name,
                avatar: "".to_string(),
                email: self.email,
                uuid: self.id,
                domain: format!("https://{}.1password.com/", "todo"), // TODO :: Requires output from op account get --format=json // WHY ?????
            }
        }
    }
}

pub mod item {
    use super::super::one_pux;
    use crate::config::runtime::RuntimeConfig;
    use crate::sources::op::account::AccountCommon;
    use chrono::{DateTime, FixedOffset};
    use lib::simplelog::trace;
    use serde::{Deserialize, Serialize};
    use serde_json::from_slice;
    use std::process::Command;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[serde(rename_all = "SCREAMING_SNAKE_CASE")]
    pub enum ItemCategory {
        Login,
        CreditCard,
        SecureNote,
        Identity,
    }

    #[derive(Debug, Clone, Deserialize)]
    pub struct Item {
        id: String,
        title: String,
        #[serde(default)]
        favorite: bool,
        /// If this is none the item is active, otherwise it is archived
        #[serde(default)]
        state: Option<String>, // TODO :: Enum
        #[serde(default)]
        tags: Vec<String>,
        version: u8, // TODO :: What is this used for
        vault: super::vault::Reference,
        category: ItemCategory,
        /// The UUID of the User which last edited this item
        last_edited_by: String,
        created_at: DateTime<FixedOffset>,
        updated_at: DateTime<FixedOffset>,
        additional_information: String,
        #[serde(default)]
        urls: Vec<super::url::Url>,
        #[serde(default)]
        sections: Vec<super::section::Section>,
        #[serde(default, deserialize_with = "super::field::deserialise")]
        fields: Vec<super::field::Field>,
    }

    impl Into<String> for ItemCategory {
        fn into(self) -> String {
            match self {
                ItemCategory::Login => "001".to_string(),
                ItemCategory::CreditCard => "002".to_string(),
                ItemCategory::SecureNote => "003".to_string(),
                ItemCategory::Identity => "004".to_string(),
            }
        }
    }

    impl Into<one_pux::item::Attrs> for Item {
        fn into(self) -> one_pux::item::Attrs {
            one_pux::item::Attrs {
                uuid: self.id,
                fav_index: if self.favorite { 1 } else { 0 },
                created_at: self.created_at.timestamp(),
                updated_at: self.updated_at.timestamp(),
                state: self.state.unwrap_or_else(|| "active".to_string()),
                category_uuid: self.category.into(),
            }
        }
    }

    impl Into<one_pux::item::Overview> for Item {
        fn into(self) -> one_pux::item::Overview {
            one_pux::item::Overview {
                title: self.title,
                subtitle: self.additional_information,
                url: self
                    .urls
                    .iter()
                    .find(|u| u.primary)
                    .map(|u| u.href.clone())
                    .unwrap_or("".to_string()),
                urls: self.urls.into_iter().map(super::url::Url::into).collect(),
                tags: match self.tags {
                    tag if tag.is_empty() => None,
                    tags => Some(tags),
                },
                icons: None,                 // TODO
                ps: 0,                       // TODO // I think this might be Password Strength
                pbe: 0.0,                    // TODO
                pgrng: false, // TODO // I think this might be to indicate that the password was generated by 1Password
                watchtower_exclusions: None, // TODO
            }
        }
    }

    impl Into<one_pux::item::Details> for Item {
        fn into(self) -> one_pux::item::Details {
            let mut fields = self.fields.clone();

            let mut password_history: Option<Vec<one_pux::item::PasswordHistory>> = None;

            let login_fields = fields
                .clone()
                .into_iter()
                .enumerate()
                .inspect(|(_, f)| trace!("Testing {:?} for login field", f))
                .filter(|(_, f)| f.is_login_field())
                .map(|(i, _)| fields.remove(i))
                .inspect(|f| {
                    // TODO :: This is a bit of a hack, but it works for now
                    if f.password_details.as_ref().is_some() {
                        let _ = password_history.insert(f.password_details.clone().unwrap().into());
                    }
                })
                .map(|f| f.into())
                .collect::<Vec<one_pux::item::Field>>();

            let notes_plain = fields
                .clone()
                .into_iter()
                .enumerate()
                .find(|(_, f)| f.is_notes_field())
                .map(|(i, _)| fields.remove(i))
                .and_then(|f| f.attrs.value)
                .unwrap_or_default();

            let sections = self
                .sections
                .into_iter()
                .map(|s| s.into(fields.clone()))
                .collect::<Vec<one_pux::item::AdditionalSection>>();

            one_pux::item::Details {
                login_fields,
                notes_plain,
                sections,
                password_history: password_history.unwrap_or_default(),
            }
        }
    }

    impl Into<one_pux::item::Item> for Item {
        // TODO -> Don't clone twice
        fn into(self) -> one_pux::item::Item {
            let attrs = self.clone().into();
            let overview = self.clone().into();
            let details = self.into();

            one_pux::item::Item {
                attrs,
                overview,
                details,
            }
        }
    }

    impl Item {
        fn raw(vault_id: &String, mut command: Command) -> Vec<u8> {
            command
                .args(&["item", "list"])
                .args(&["--vault", &vault_id, "--format=json"])
                .output()
                .unwrap()
                .stdout
        }

        fn raw_long(vault_id: &String, item_id: &String, mut command: Command) -> Vec<u8> {
            command
                .args(&["item", "get", &item_id])
                .args(&["--vault", &vault_id, "--format=json"])
                .output()
                .unwrap()
                .stdout
        }

        pub fn parse(
            vault: super::vault::Vault,
            account: &Box<&dyn AccountCommon>,
            config: &RuntimeConfig,
        ) -> Vec<Item> {
            trace!(
                "Requesting Items from {}-{}",
                &vault.reference.name,
                &vault.reference.id
            );

            let raw = Self::raw(&vault.reference.id, account.command(&config));
            trace!("Raw Items JSON {}", String::from_utf8_lossy(&raw));
            let parsed = from_slice::<Vec<Item>>(&raw).unwrap();
            trace!("Parsed Items {:?}", parsed);
            parsed
                .into_iter()
                .map(|item| Self::raw_long(&vault.reference.id, &item.id, account.command(&config)))
                .map(|raw| {
                    trace!("Raw Item JSON {}", String::from_utf8_lossy(&raw));
                    let parsed = from_slice::<Item>(&raw).unwrap();
                    trace!("Parsed Item {:?}", parsed);
                    parsed
                })
                .collect()
        }
    }
}

pub mod vault {
    use crate::config::runtime::RuntimeConfig;
    use crate::sources::op::account::AccountCommon;
    use crate::sources::op::one_pux;
    use chrono::{DateTime, FixedOffset};
    use lib::simplelog::trace;
    use serde::{Deserialize, Serialize};
    use serde_json::from_slice;
    use std::process::Command;

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
        pub content_version: u32,
        pub attribute_version: u8,
        pub items: usize,
        #[serde(rename = "type")]
        pub vault_type: Type,
        pub created_at: DateTime<FixedOffset>,
        pub updated_at: DateTime<FixedOffset>,
    }

    impl Vault {
        fn raw(vault_id: &String, mut command: Command) -> Vec<u8> {
            command.args(&["vault", "get", &vault_id, "--format=json"]).output().unwrap().stdout
        }

        pub fn parse(account: &Box<&dyn AccountCommon>, config: &RuntimeConfig) -> Vec<Vault> {
            account
                .vaults()
                .into_iter()
                .inspect(|vault| trace!("Requesting Vault {}-{}", &vault.name, &vault.id))
                .map(|reference| Self::raw(&reference.id, account.command(&config)))
                .inspect(|output| trace!("Parsing Vault JSON {}", String::from_utf8_lossy(&output)))
                .map(|output| from_slice::<Vault>(output.as_slice()).unwrap())
                .collect()
        }
    }

    impl Into<one_pux::vault::Type> for Type {
        fn into(self) -> one_pux::vault::Type {
            match self {
                Type::Personal => one_pux::vault::Type::P,
                Type::Shared => one_pux::vault::Type::E,
                Type::UserCreated => one_pux::vault::Type::U,
            }
        }
    }

    impl Into<one_pux::vault::Attrs> for Vault {
        fn into(self) -> one_pux::vault::Attrs {
            one_pux::vault::Attrs {
                uuid: self.reference.id,
                desc: "".to_string(),   // TODO
                avatar: "".to_string(), // TODO
                name: self.reference.name,
                vault_type: self.vault_type.into(),
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

    impl Into<one_pux::item::UrlObject> for Url {
        fn into(self) -> one_pux::item::UrlObject {
            one_pux::item::UrlObject {
                url: self.href,
                label: self.label.unwrap_or("".to_string()),
                mode: "default".to_string(), // Unable to get from CLI
            }
        }
    }
}

pub mod field {
    use super::super::one_pux;
    use lib::simplelog::debug;
    use serde::de::{SeqAccess, Visitor};
    use serde::{Deserialize, Deserializer, Serialize};
    use std::fmt;

    #[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
    #[serde(rename_all = "UPPERCASE")]
    pub enum Type {
        String,
        Concealed,
        SshKey,
        Email,
        OTP,
    }

    #[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
    #[serde(rename_all = "UPPERCASE")]
    pub enum Purpose {
        Username,
        Password,
        Notes,
    }

    #[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
    pub struct PasswordDetails {
        pub strength: String,
        #[serde(default)]
        pub history: Vec<String>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
    pub struct FieldAttrs {
        pub id: String,
        #[serde(default)]
        pub section: Option<super::section::Section>,
        #[serde(rename = "type")]
        pub field_type: Type,
        #[serde(default)]
        pub purpose: Option<Purpose>,
        pub label: String,
        #[serde(default)]
        pub value: Option<String>,
        pub reference: String,
    }

    #[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
    pub struct Field {
        #[serde(flatten)]
        pub attrs: FieldAttrs,
        #[serde(default)]
        pub password_details: Option<PasswordDetails>,
    }

    impl Field {
        const LOGIN_PURPOSES: [Purpose; 2] = [Purpose::Username, Purpose::Password];
        const LOGIN_TYPES: [Type; 2] = [Type::String, Type::Concealed];

        pub fn is_login_field(&self) -> bool {
            match self {
                s if s.attrs.section.as_ref().is_some() => false,
                s if !Self::LOGIN_TYPES.contains(&s.attrs.field_type) => false,
                s if s.attrs.purpose.as_ref().is_none() => false,
                s if !Self::LOGIN_PURPOSES.contains(&s.attrs.purpose.as_ref().unwrap()) => false,
                _ => true,
            }
        }

        pub fn is_notes_field(&self) -> bool {
            match self {
                s if s.attrs.section.as_ref().is_some() => false,
                s if &s.attrs.field_type != &Type::String => false,
                s if s.attrs.purpose.as_ref().is_none() => false,
                s if s.attrs.purpose.as_ref().unwrap() != &Purpose::Notes => false,
                s if s.attrs.value.is_none() => false,
                _ => true,
            }
        }
    }

    // /// Currently if the field doesn't have a purpose it is ignored,
    // /// This only affects secondary usernames and passwords but it still needs to be fixed
    // #[derive(Debug, Clone)]
    // // #[serde(tag = "purpose", rename_all = "UPPERCASE")]
    // pub enum Field {
    //     Username {
    //         // #[serde(flatten)]
    //         common: CommonField,
    //     },
    //     Password {
    //         // #[serde(flatten)]
    //         common: CommonField,
    //         password_details: PasswordDetails, // ??
    //     },
    //     Notes {
    //         // #[serde(flatten)]
    //         common: CommonField,
    //     },
    //     TOTP {
    //         // #[serde(flatten)]
    //         common: CommonField,
    //     },
    // }
    //
    // impl Field {
    //     pub fn get(&self) -> &CommonField {
    //         match self {
    //             Field::Username { common } => common,
    //             Field::Password { common, .. } => common,
    //             Field::Notes { common } => common,
    //             Field::TOTP { common } => common,
    //         }
    //     }
    // }

    impl Into<one_pux::item::FieldType> for Type {
        fn into(self) -> one_pux::item::FieldType {
            match self {
                Type::String => one_pux::item::FieldType::Text,
                Type::Concealed => one_pux::item::FieldType::Password,
                Type::SshKey => one_pux::item::FieldType::Text, // TODO
                Type::Email => one_pux::item::FieldType::Email, // TODO
                Type::OTP => one_pux::item::FieldType::Text,    // TODO // Is this right?
            }
        }
    }

    impl Into<one_pux::item::FieldDesignation> for Purpose {
        fn into(self) -> one_pux::item::FieldDesignation {
            match self {
                Purpose::Username => one_pux::item::FieldDesignation::Username,
                Purpose::Password => one_pux::item::FieldDesignation::Password,
                Purpose::Notes => one_pux::item::FieldDesignation::None,
            }
        }
    }

    impl Into<one_pux::item::Field> for Field {
        fn into(self) -> one_pux::item::Field {
            let common = self.attrs.clone();
            one_pux::item::Field {
                id: common.id,
                name: common.label,
                value: common.value.unwrap_or("".to_string()),
                designation: common
                    .purpose
                    .map(|p| p.into())
                    .unwrap_or(one_pux::item::FieldDesignation::None),
                field_type: common.field_type.into(),
            }
        }
    }

    impl Into<Vec<one_pux::item::PasswordHistory>> for PasswordDetails {
        fn into(self) -> Vec<one_pux::item::PasswordHistory> {
            self.history
                .iter()
                .map(|h| one_pux::item::PasswordHistory {
                    value: h.clone(),
                    time: 0, // TODO -> I'm unsure if the cli can expose this
                })
                .collect::<Vec<one_pux::item::PasswordHistory>>()
        }
    }

    // TODO :: SSHKey support
    impl Into<one_pux::section::Field> for Field {
        fn into(self) -> one_pux::section::Field {
            let common = self.attrs.clone();

            one_pux::section::Field {
                title: common.label,
                id: common.id,
                value: match common.field_type {
                    Type::String => {
                        one_pux::section::Value::String(common.value.unwrap_or_default())
                    }
                    Type::Concealed => {
                        one_pux::section::Value::Concealed(common.value.unwrap_or_default())
                    }
                    Type::SshKey => one_pux::section::Value::SshKey {
                        private_key: "".to_string(),
                        metadata: one_pux::section::SshKeyMetadata {
                            private_key: "".to_string(),
                            public_key: "".to_string(),
                            fingerprint: "".to_string(),
                            key_type: "".to_string(),
                        },
                    },
                    Type::Email => one_pux::section::Value::Email {
                        email_address: common.value.unwrap_or_default(),
                        provider: None,
                    },
                    Type::OTP => one_pux::section::Value::TOTP(common.value.unwrap_or_default()),
                },
                guarded: false,
                multiline: false,
                dont_generate: false,
                input_traits: one_pux::section::InputTraits::default(),
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

            // fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            //     where
            //         A: MapAccess<'de>,
            // {
            //     info!("Visiting map");
            //     while let Some(key) = map.next_key::<String>()? {
            //         info!("Visiting key: {}", key);
            //         info!("Visiting value: {}", map.next_value::<String>()?);
            //     }
            //
            //     Err(Error::custom("Not implemented map"))
            // }
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
        pub fn into(self, fields: Vec<super::field::Field>) -> one_pux::item::AdditionalSection {
            one_pux::item::AdditionalSection {
                title: self.id.clone(),
                name: self.label.unwrap_or_default(),
                fields: fields
                    .into_iter()
                    .filter(|f| f.attrs.section.clone().is_some_and(|s| &s.id == &self.id))
                    .map(|f| f.into())
                    .collect(),
            }
        }
    }
}
