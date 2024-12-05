/*
 * Copyright (C) 2023-2024. James Draycott me@racci.dev
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, version 3.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.
 * See the GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program. If not, see https://www.gnu.org/licenses/.
 */

pub mod identifier {
    use serde::{Deserialize, Serialize};
    use std::fmt::{Display, Formatter};

    #[cfg(test)]
    use fake::{faker::lorem::en::Word, Dummy};

    /// The internal identifier for this entity.
    #[cfg_attr(test, derive(Dummy))]
    #[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
    pub struct Id(
        #[serde(default, skip_serializing_if = "String::is_empty")]
        #[cfg_attr(test, dummy(faker = "26"))]
        pub String,
    );

    /// The user facing & modifiable name for this entity.
    #[cfg_attr(test, derive(Dummy))]
    #[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
    #[serde(rename_all = "lowercase")]
    pub enum UniqueName {
        Label(
            #[serde(default, skip_serializing_if = "String::is_empty")]
            #[cfg_attr(test, dummy(faker = "Word()"))]
            String,
        ),
        Name(
            #[serde(default, skip_serializing_if = "String::is_empty")]
            #[cfg_attr(test, dummy(faker = "Word()"))]
            String,
        ),
        Title(
            #[serde(default, skip_serializing_if = "String::is_empty")]
            #[cfg_attr(test, dummy(faker = "Word()"))]
            String,
        ),
    }

    #[cfg_attr(test, derive(Dummy))]
    #[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
    pub struct Identifier {
        pub id: Id,
        #[serde(flatten)]
        pub unique_name: UniqueName,
    }

    impl Identifier {
        /// # Returns
        /// The internal tracking identifier for this entity.
        pub fn id(&self) -> &str {
            self.id.0.as_str()
        }

        /// # Returns
        /// The user facing & modifiable name for this entity.
        pub fn named(&self) -> &str {
            match &self.unique_name {
                UniqueName::Label(label, ..) => label,
                UniqueName::Name(name, ..) => name,
                UniqueName::Title(title, ..) => title,
            }
        }
    }

    impl Display for Identifier {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            write!(f, "{named} ({id})", named = &self.named(), id = &self.id())
        }
    }
}

pub mod state {
    use serde::{Deserialize, Serialize};
    use std::fmt::{Display, Formatter};

    #[cfg(test)]
    use fake::Dummy;

    // TODO -> Are there any other states?
    #[cfg_attr(test, derive(Dummy))]
    #[derive(Default, Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
    #[serde(rename_all = "SCREAMING_SNAKE_CASE")]
    pub enum State {
        #[default]
        Active,
        Archived,
    }

    impl Display for State {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            write!(f, "{self:?}")
        }
    }
}

pub mod dated {
    use chrono::{DateTime, Utc};
    use serde::{Deserialize, Serialize};
    use std::fmt::{Display, Formatter};

    #[cfg(test)]
    use {
        fake::{faker::chrono::en::DateTimeAfter, Dummy},
        std::time::UNIX_EPOCH,
    };

    #[cfg_attr(test, derive(Dummy))]
    #[derive(Default, Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
    pub struct Dated {
        #[cfg_attr(test, dummy(faker = "DateTimeAfter(DateTime::<Utc>::from(UNIX_EPOCH))"))]
        pub created_at: DateTime<Utc>,
        #[cfg_attr(test, dummy(faker = "DateTimeAfter(created_at)"))]
        pub updated_at: DateTime<Utc>,
    }

    impl Display for Dated {
        // TODO -> Relative time?
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            write!(
                f,
                "[C@{created_at} - U@{updated_at}]",
                created_at = &self.created_at.format("%Y-%m-%d"),
                updated_at = &self.updated_at.format("%Y-%m-%d")
            )
        }
    }
}

pub mod user {
    use crate::sources::getter::CliGetter;
    use crate::sources::op::core::OnePasswordCore;
    use chrono::{DateTime, Utc};
    use serde::{Deserialize, Serialize};

    #[cfg(test)]
    use fake::{
        faker::{chrono::en::DateTimeBetween, internet::en::FreeEmail},
        Dummy,
    };
    use macros::CommonFields;

    #[cfg_attr(test, derive(Dummy))]
    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    pub struct Attrs {
        #[serde(flatten)]
        pub identifier: super::identifier::Identifier,

        /// The users email address.
        #[cfg_attr(test, dummy(faker = "FreeEmail()"))]
        pub email: String,

        /// The account state of the user.
        pub state: super::state::State,

        /// The date information for the user.
        #[serde(flatten)]
        pub dated: super::dated::Dated,

        /// When this user was last authenticated with the cli.
        #[cfg_attr(test, dummy(faker = "DateTimeBetween(dated.created_at, dated.updated_at)"))]
        pub last_auth_at: DateTime<Utc>,
    }

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize, CommonFields)]
    #[serde(rename_all = "SCREAMING_SNAKE_CASE", tag = "type")]
    pub enum User {
        Member {
            #[serde(flatten)]
            attrs: Attrs,
        },
        ServiceAccount {
            #[serde(flatten)]
            attrs: Attrs,
        },
    }

    impl CliGetter<OnePasswordCore, User, [&'static str; 3]> for User {
        const ARGS: [&'static str; 3] = ["user", "get", "--me"];
    }

    #[test]
    fn test_serialisation() {
        let json = r#"
{
    "type": "MEMBER",
    "id": "LAIQMSG1PWNMCA9LAS5KLSDURN",
    "name": "Test Name",
    "email": "test@gmail.com",
    "state": "ACTIVE",
    "created_at": "2023-01-28T06:14:27Z",
    "updated_at": "2023-01-28T06:15:18Z",
    "last_auth_at": "2023-06-23T08:14:56Z"
}
        "#
        .trim();

        let user = serde_json::from_str::<User>(json).unwrap();
        let serialised = serde_json::to_string_pretty(&user).unwrap();

        assert_eq!(json, serialised, "Serialisation should be the same");
    }
}

pub mod whoami {
    use crate::sources::getter::CliGetter;
    use crate::sources::op::core::OnePasswordCore;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[serde(rename_all = "SCREAMING_SNAKE_CASE")]
    pub enum WhoAmI {
        User {
            #[serde(flatten)]
            short: super::account::AccountShort,
        },
        ServiceAccount {
            #[serde(rename = "URL")]
            url: String,
            #[serde(rename = "serviceAccountType")]
            service_account_type: String, // TODO -> Enum?
        },
    }

    impl CliGetter<OnePasswordCore, WhoAmI, [&'static str; 1]> for WhoAmI {
        const ARGS: [&'static str; 1] = ["whoami"];
    }
}

pub mod account {
    use crate::sources::getter::CliGetter;
    use crate::sources::op::core::OnePasswordCore;
    use crate::sources::op::one_pux;
    use chrono::{DateTime, Utc};
    use serde::{Deserialize, Serialize};

    use crate::sources::op::cli::identifier;
    use macros::CommonFields;
    #[cfg(test)]
    use {
        fake::{
            faker::{
                chrono::en::DateTimeAfter,
                internet::en::{DomainSuffix, FreeEmail},
            },
            Dummy,
        },
        std::time::UNIX_EPOCH,
    };

    /// This comes from the list of account gotten with `op list accounts`
    #[cfg_attr(test, derive(Dummy))]
    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    pub struct AccountShort {
        #[cfg_attr(test, dummy(faker = "DomainSuffix()"))]
        pub url: String,

        #[cfg_attr(test, dummy(faker = "FreeEmail()"))]
        pub email: String,

        #[cfg_attr(test, dummy(faker = "26"))]
        pub user_uuid: String,

        #[cfg_attr(test, dummy(faker = "26"))]
        pub account_uuid: String,
    }

    #[cfg_attr(test, derive(Dummy))]
    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    pub struct Attrs {
        #[serde(flatten)]
        pub identifier: identifier::Identifier,

        /// The 1password domain for this account
        /// ### This is only the subdomain and not the entire url.
        #[cfg_attr(test, dummy(faker = "DomainSuffix()"))]
        pub domain: String,

        /// The accounts current state.
        pub state: super::state::State,

        /// When this account was created.
        #[cfg_attr(test, dummy(faker = "DateTimeAfter(DateTime::<Utc>::from(UNIX_EPOCH))"))]
        pub created_at: DateTime<Utc>,
    }

    #[cfg_attr(test, derive(Dummy))]
    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize, CommonFields)]
    #[serde(rename_all = "SCREAMING_SNAKE_CASE", tag = "type")]
    pub enum Account {
        Individual {
            #[serde(flatten)]
            attrs: Attrs,

            #[serde(default, skip)]
            short: Option<AccountShort>,
        },
        Business {
            #[serde(flatten)]
            attrs: Attrs,

            #[serde(default, skip)]
            short: Option<AccountShort>,
        },
    }

    impl CliGetter<OnePasswordCore, Vec<AccountShort>, [&'static str; 2]> for AccountShort {
        const ARGS: [&'static str; 2] = ["account", "list"];
    }

    impl CliGetter<OnePasswordCore, Account, [&'static str; 2]> for Account {
        const ARGS: [&'static str; 2] = ["account", "get"];
    }

    impl From<Account> for one_pux::account::Attrs {
        fn from(value: Account) -> one_pux::account::Attrs {
            let attrs = value.attrs().to_owned();
            let short = value.short();

            one_pux::account::Attrs {
                account_name: attrs.identifier.named().to_owned(),
                name: attrs.identifier.named().to_owned(),
                avatar: "".to_string(), // TODO -> Unsure if this is available // Is a filename from the archive
                email: short.clone().map(|s| s.email.to_owned()).unwrap_or_default(),
                uuid: attrs.identifier.id().to_owned(),
                domain: format!("https://{}.1password.com/", value.attrs().domain),
            }
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use fake::{Fake, Faker};
        use serde_json::json;
        use tracing::debug;

        #[test]
        fn test_short() {
            let reference: AccountShort = Faker.fake();

            let json = json!({
                "url": reference.url,
                "email": reference.email,
                "user_uuid": reference.user_uuid,
                "account_uuid": reference.account_uuid,
            })
            .to_string();

            let short = serde_json::from_str::<AccountShort>(&json).unwrap();
            assert_eq!(short.url, reference.url);
            assert_eq!(short.email, reference.email);
            assert_eq!(short.user_uuid, reference.user_uuid);
            assert_eq!(short.account_uuid, reference.account_uuid);
        }

        #[test_log::test(test)]
        fn test_deserialisation() -> anyhow::Result<()> {
            let reference = Faker.fake::<Account>();

            let json = json!({
                "type": "INDIVIDUAL",
                "id": reference.attrs().identifier.id(),
                "name": reference.attrs().identifier.named(),
                "domain": reference.attrs().domain,
                "state": reference.attrs().state,
                "created_at": reference.attrs().created_at,
            });

            let account = serde_json::from_value::<Account>(json)?;
            debug!("{account:#?}");
            let attrs = account.attrs();

            assert_eq!(
                attrs.identifier.id(),
                reference.attrs().identifier.id(),
                "Id's should be the same"
            );
            assert_eq!(
                attrs.identifier.named(),
                reference.attrs().identifier.named(),
                "Names should be the same"
            );
            assert_eq!(attrs.domain, reference.attrs().domain, "Domains should be the same");
            assert_eq!(attrs.state, reference.attrs().state, "States should be the same");
            assert_eq!(
                attrs.created_at,
                reference.attrs().created_at,
                "Created at should be the same"
            );

            Ok(())
        }
    }
}

pub mod file {
    use crate::sources::op::cli::dated::Dated;
    use crate::sources::op::cli::{identifier, vault};
    use crate::sources::op::one_pux;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Reference {
        /// Name Identifier.
        #[serde(flatten)]
        pub identifier: identifier::Identifier,

        pub size: usize,

        pub content_path: String,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Attrs {
        /// Title identifier.
        #[serde(flatten)]
        pub identifier: identifier::Identifier,

        /// The times this file has been modified.
        pub version: String,

        /// A reference to the vault which contains this file.
        pub vault: vault::Reference,

        /// The file's size in a human readable format.
        #[serde(rename = "overview.ainfo")]
        pub info: String,

        /// The uuid of the user who last edited this file.
        pub last_edited_by: String,

        #[serde(flatten)]
        pub dated: Dated,
    }

    impl From<Reference> for one_pux::item::DocumentAttributes {
        fn from(value: Reference) -> Self {
            one_pux::item::DocumentAttributes {
                file_name: value.identifier.named().to_owned(),
                document_id: value.identifier.id().to_owned(),
                decrypted_size: value.size,
            }
        }
    }

    impl Reference {
        #[allow(dead_code)]
        pub(crate) fn new(id: &str, name: &str, size: usize, content_path: &str) -> Self {
            Self {
                identifier: identifier::Identifier {
                    id: identifier::Id(id.into()),
                    unique_name: identifier::UniqueName::Name(name.into()),
                },
                size,
                content_path: content_path.to_owned(),
            }
        }
    }

    impl Attrs {
        #[allow(dead_code)]
        pub(crate) fn new(
            id: &str,
            title: &str,
            version: &str,
            vault: vault::Reference,
            info: &str,
            last_edited_by: &str,
            dated: Dated,
        ) -> Self {
            Self {
                identifier: identifier::Identifier {
                    id: identifier::Id(id.to_owned()),
                    unique_name: identifier::UniqueName::Title(title.to_owned()),
                },
                version: version.to_owned(),
                vault,
                info: info.to_owned(),
                last_edited_by: last_edited_by.to_owned(),
                dated,
            }
        }
    }
}

// TODO -> Items may be a entire enum type per category with category being the enum variant
pub mod item {
    use super::super::one_pux;
    use crate::config::runtime::Runtime;
    use crate::sources::op::account::OnePasswordAccount;
    use anyhow::{Context, Result};
    use indicatif::{MultiProgress, ParallelProgressIterator, ProgressBar};
    use macros::CommonFields;
    use rayon::prelude::*;
    use serde::{Deserialize, Serialize};
    use serde_json::from_slice;
    use std::fmt::{Display, Formatter};
    use std::process::Command;
    use tracing::{error, instrument, trace};

    #[derive(Debug, Clone, Serialize, Deserialize, CommonFields)]
    #[serde(rename_all = "SCREAMING_SNAKE_CASE", tag = "category")]
    pub enum Item {
        /// Additional is the value of the field `username`
        Login {
            #[serde(flatten)]
            attrs: Attrs,
        },
        /// An explicit password without a username, why the fuck does this exist???????
        Password {
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
        // TODO -> Very much broken currently, will block import
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

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Attrs {
        #[serde(flatten)]
        pub identifier: super::identifier::Identifier,

        /// If this user has marked this item as a favorite
        #[serde(default)] // TODO -> Only serialize if true
        pub favorite: bool,

        /// The tags associated with this item, if any.
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        pub tags: Vec<String>,

        /// The number of times this item has been changed.
        /// Items start at 1.
        pub version: usize,

        /// The state of the item, either active or archived.
        #[serde(default)]
        pub state: super::state::State,

        /// A reference to the vault which owns this item.
        pub vault: super::vault::Reference,

        /// The UUID of the User which last edited this item
        pub last_edited_by: String,

        #[serde(flatten)]
        pub dated: super::dated::Dated,

        // TODO :: Seems to be category specific
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub additional_information: Option<String>,

        /// The URLs associated with this item, if any.
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        pub urls: Vec<super::url::Url>,

        /// The additional sections of this item, if any.
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        pub sections: Vec<super::section::Section>,

        /// The fields associated with this item, if any.
        #[serde(default, deserialize_with = "super::field::deserialise")]
        pub fields: Vec<super::field::Field>,
    }

    impl From<Item> for one_pux::item::Attrs {
        fn from(val: Item) -> Self {
            let attrs = val.attrs().clone();

            one_pux::item::Attrs {
                uuid: attrs.identifier.id().to_owned(),
                fav_index: attrs.favorite.into(),
                created_at: attrs.dated.created_at.timestamp(),
                updated_at: attrs.dated.updated_at.timestamp(),
                state: attrs.state.to_string().to_lowercase(),
                category_uuid: val.get_category_id(),
            }
        }
    }

    impl From<Item> for Option<one_pux::item::PasswordDetails> {
        fn from(value: Item) -> Option<one_pux::item::PasswordDetails> {
            let attrs = value.attrs();
            let field = attrs
                .fields
                .clone()
                .into_iter()
                .filter_map(|f| match f {
                    super::field::Field::Concealed { password_details, .. } => Some(password_details),
                    _ => None,
                })
                .flatten()
                .next();

            match field {
                None => None,
                Some(details) => Some(one_pux::item::PasswordDetails {
                    password_strength: details.strength.map(|s| s.into()).unwrap_or_default(),
                    password_base_entropy: details.entropy as f64,
                    password_generated: details.generated,
                }),
            }
        }
    }

    impl From<Item> for one_pux::item::Overview {
        fn from(val: Item) -> Self {
            let attrs = val.attrs().clone();
            let watchtower_exclusions: Option<one_pux::item::WatchTowerExclusions> = None; // TODO
            let password_details: Option<one_pux::item::PasswordDetails> = val.into();

            one_pux::item::Overview {
                subtitle: attrs.additional_information.unwrap_or_default(),
                icons: None, // TODO
                urls: attrs.urls.clone().into_iter().map(super::url::Url::into).collect(),
                tags: attrs.tags,
                title: attrs.identifier.named().to_owned(),
                url: attrs.urls.into_iter().find(|u| u.primary).map(|u| u.href).unwrap_or_default(),
                password_details,
                watchtower_exclusions,
            }
        }
    }

    impl TryFrom<Item> for one_pux::item::Details {
        type Error = anyhow::Error; // TODO -> ThisError

        fn try_from(value: Item) -> Result<one_pux::item::Details> {
            let mut fields = value.attrs().fields.clone();

            let mut password_history = None;
            let mut removed = 0usize;
            let login_fields = fields
                .clone()
                .into_iter()
                .enumerate()
                .filter(|(_, f)| f.is_login_field())
                .inspect(|(_, f)| {
                    if let super::field::Field::Concealed { password_details, .. } = f {
                        if password_details.is_some() {
                            let into = password_details
                                .clone() // TODO -> Better error handling
                                .unwrap_or_else(|| panic!("Get password details of {value}"))
                                .into();
                            let _ = password_history.insert(into);
                        }
                    }
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
                .and_then(|f| f.attrs().value.clone());

            // TODO -> This is a bit of a hack, but it works for now
            // TODO -> Sections aren't in the right order, but that's not a big deal
            let mut sections = value
                .attrs()
                .sections
                .clone()
                .into_iter()
                .map(|s| s.into())
                .collect::<Vec<one_pux::item::AdditionalSection>>();
            for field in fields {
                let attrs = field.attrs();

                if attrs.section.as_ref().is_none() {
                    if sections.is_empty() || !sections.iter().any(|s| s.title.is_empty() && s.name.is_empty()) {
                        sections.insert(0, one_pux::item::AdditionalSection::default())
                    }

                    sections
                        .first_mut()
                        .context("Get empty section for fields.")?
                        .fields
                        .push(field.into());
                    continue;
                }

                let section_ref = attrs
                    .section
                    .as_ref()
                    .with_context(|| format!("Get section for field {} of item {value}.", attrs.identifier,))?;

                if let Some(s) = sections.iter_mut().find(|s| s.name == section_ref.id()) {
                    s.fields.push(field.into())
                }
            }

            let document_attributes = match value {
                Item::Document { files, .. } => Some(files.first().map(|f| f.clone().into())),
                _ => None,
            }
            .flatten();

            Ok(one_pux::item::Details {
                login_fields,
                notes_plain,
                sections,
                password_history: password_history.context("Unwrap password history").unwrap_or_default(),
                document_attributes,
            })
        }
    }

    impl TryFrom<Item> for one_pux::item::Item {
        type Error = anyhow::Error; // TODO -> ThisError

        // TODO -> Don't clone twice
        fn try_from(value: Item) -> Result<one_pux::item::Item> {
            let attrs = value.clone().into();
            let overview = value.clone().into();
            let details = value.try_into()?;

            Ok(one_pux::item::Item {
                attrs,
                overview,
                details,
            })
        }
    }

    impl Item {
        pub fn get_category_id(&self) -> String {
            match self {
                Item::Login { .. } => "001",
                Item::CreditCard { .. } => "002",
                Item::SecureNote { .. } => "003",
                Item::Identity { .. } => "004",
                Item::Password { .. } => "005", // TODO -> Double check this is correct
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

        fn raw(vault_id: &str, mut command: Command) -> Result<Vec<u8>> {
            command
                .args(["item", "list"])
                .args([/*"--include-archive",*/ "--vault", vault_id, "--format=json"]) // TODO -> Add archive support
                .output()
                .with_context(|| format!("Get item list from vault {vault_id}."))
                .inspect(|o| {
                    if !o.status.success() {
                        error!("Failed to get item list from vault {vault_id}.");
                        error!("stdout: {}", String::from_utf8_lossy(&o.stdout));
                        error!("stderr: {}", String::from_utf8_lossy(&o.stderr));
                    }
                })
                .map(|o| o.stdout)
        }

        fn raw_long(vault: &super::vault::Reference, item: Item, mut command: Command) -> Result<Vec<u8>> {
            command
                .args(["item", "get", &item.attrs().identifier.id()])
                .args(["--vault", &vault.id(), "--format=json"])
                .output()
                .with_context(|| format!("Get item {item} from vault {vault}."))
                .inspect(|o| {
                    if !o.status.success() {
                        error!("Failed to get long item {item} from vault {vault}.");
                        error!("stdout: {}", String::from_utf8_lossy(&o.stdout));
                        error!("stderr: {}", String::from_utf8_lossy(&o.stderr));
                    }
                })
                .map(|o| o.stdout)
        }

        #[instrument(level = "TRACE")]
        pub fn parse(
            vault: super::vault::Vault,
            account: &OnePasswordAccount,
            config: &Runtime,
            bars: (&ProgressBar, &MultiProgress),
        ) -> Result<Vec<Item>> {
            trace!("Requesting Items from {vault}");
            let bar = bars.1.insert_after(bars.0, amt_lib::ui::cli::progress::spinner_with_count());

            bar.set_message(format!("Requesting items from `{vault}` vault...",));

            let items = Self::raw(vault.attrs().reference.id(), account.command(config)?)
                .and_then(|raw| from_slice::<Vec<Item>>(&raw).context("Deserialize items list"))?;

            bar.set_length(items.len() as u64);
            bar.set_message(format!("Requesting items details from `{vault}` vault...",));

            let items = items
                .into_par_iter()
                .progress_with(bar)
                .map(|item| Self::raw_long(&vault.attrs().reference, item, account.command(config)?))
                .map(|r| {
                    if r.is_err() {
                        return Err(r.err().unwrap());
                    }

                    let vec = r.unwrap();
                    from_slice::<Item>(vec.as_slice()).context("Deserialize item")
                })
                .collect::<Vec<Result<Item>>>();

            let mut fin = vec![];
            for item in items {
                fin.push(item?)
            }

            Ok(fin)
        }
    }

    impl Display for Item {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            write!(
                f,
                "{}-{}",
                self.attrs().identifier.id(),
                self.attrs().identifier.named()
            )
        }
    }

    #[cfg(test)]
    mod test {
        use super::*;
        use crate::sources::op::cli::{dated, identifier, state, vault};
        use chrono::{DateTime, Utc};
        use fake::faker::boolean::en::Boolean;
        use fake::faker::chrono::en::DateTimeAfter;
        use fake::faker::lorem::en::Word;
        use fake::{Fake, Faker};
        use rand::random;
        use serde_json::{from_str, json};
        use std::time::UNIX_EPOCH;

        fn random_id() -> String {
            26.fake::<String>()
        }

        fn random_dated() -> dated::Dated {
            let epoch = DateTime::<Utc>::from(UNIX_EPOCH);
            let created_at = DateTimeAfter(epoch).fake();
            let updated_at = DateTimeAfter(created_at).fake();

            dated::Dated { created_at, updated_at }
        }

        fn random_defaults() -> Attrs {
            Attrs {
                identifier: Faker.fake(),
                favorite: Boolean(5).fake(),
                tags: fake::vec![String as Word(); 1..3],
                version: (1..50).fake::<usize>(),
                state: match random() {
                    true => state::State::Archived,
                    false => state::State::Active,
                },
                vault: vault::Reference {
                    id: Faker.fake(),
                    unique_name: identifier::UniqueName::Name(Faker.fake()),
                },
                last_edited_by: random_id(),
                dated: random_dated(),
                additional_information: None,
                sections: vec![],
                urls: vec![],
                fields: vec![],
            }
        }

        fn json_item(item: Item) -> String {
            let attrs = item.attrs();
            json!({
                "id": attrs.identifier.id(),
                "title": attrs.identifier.named(),
                "tags": attrs.tags,
                "version": attrs.version,
                "vault": {
                    "id": attrs.vault.id(),
                    "name": attrs.vault.named()
                },
                "category": match item {
                    Item::Login { .. } => "LOGIN",
                    Item::CreditCard { .. } => "CREDIT_CARD",
                    Item::SecureNote { .. } => "SECURE_NOTE",
                    Item::Identity { .. } => "IDENTITY",
                    Item::Password { .. } => "PASSWORD",
                    Item::Document { .. } => "DOCUMENT",
                    Item::SoftwareLicense { .. } => "SOFTWARE_LICENSE",
                    Item::Database { .. } => "DATABASE",
                    Item::DriverLicense { .. } => "DRIVER_LICENSE",
                    Item::OutdoorLicense { .. } => "OUTDOOR_LICENSE",
                    Item::Membership { .. } => "MEMBERSHIP",
                    Item::Passport { .. } => "PASSPORT",
                    Item::RewardProgram { .. } => "REWARD_PROGRAM",
                    Item::WirelessRouter { .. } => "WIRELESS_ROUTER",
                    Item::Server { .. } => "SERVER",
                    Item::EmailAccount { .. } => "EMAIL_ACCOUNT",
                    Item::ApiCredential { .. } => "API_CREDENTIAL",
                    Item::MedicalRecord { .. } => "MEDICAL_RECORD",
                    Item::SshKey { .. } => "SSH_KEY",
                    Item::Custom { .. } => "CUSTOM",
                },
                "last_edited_by": attrs.last_edited_by,
                "created_at": attrs.dated.created_at,
                "updated_at": attrs.dated.updated_at,
                "additional_information": attrs.additional_information,
                "sections": attrs.sections,
                "fields": attrs.fields,
            })
            .to_string()
        }

        #[test]
        fn test_common_attrs() -> Result<()> {
            let attrs = random_defaults();
            let json = json_item(Item::Login { attrs: attrs.clone() });

            println!("json: {json:#}");

            let item = from_str::<Item>(&json);
            assert!(
                item.is_ok(),
                "Item should deserialize without error; {:#?}",
                item.err().unwrap()
            );
            let item = item?;

            assert_eq!(item.attrs().identifier.id(), attrs.identifier.id());
            assert_eq!(item.attrs().identifier.named(), attrs.identifier.named());
            assert_eq!(item.attrs().tags, attrs.tags);
            assert_eq!(item.attrs().version, attrs.version);
            assert_eq!(item.attrs().vault.id(), attrs.vault.id());
            assert_eq!(item.attrs().vault.named(), attrs.vault.named());
            assert_eq!(item.attrs().last_edited_by, attrs.last_edited_by);
            assert_eq!(item.attrs().dated.created_at, attrs.dated.created_at);
            assert_eq!(item.attrs().dated.updated_at, attrs.dated.updated_at);

            Ok(())
        }

        #[test]
        fn test_login_item() {
            let attrs = random_defaults();
            let fields = super::super::field::test::random_login(&attrs);

            let reference_item = Item::Login {
                attrs: Attrs {
                    additional_information: fields[0].attrs().clone().value,
                    fields: fields.to_vec(),
                    ..attrs
                },
            };

            let item_json = json_item(reference_item.clone());
            println!("Item JSON: {item_json}");

            let item = from_str::<Item>(&item_json);
            assert!(
                item.is_ok(),
                "Item should deserialize without error; {:#?}",
                item.err().unwrap()
            );
            let _item = item.unwrap();
        }
    }
}

pub mod vault {
    use crate::config::runtime::Runtime;
    use crate::sources::getter::CliGetter;
    use crate::sources::op::account::OnePasswordAccount;
    use crate::sources::op::cli::dated::Dated;
    use crate::sources::op::cli::identifier::Identifier;
    use crate::sources::op::core::OnePasswordCore;
    use crate::sources::op::one_pux;
    use anyhow::Result;
    #[cfg(test)]
    use fake::Dummy;
    use macros::CommonFields;
    use serde::{Deserialize, Serialize};
    use std::fmt::Display;
    use tracing::{trace, warn};

    pub type Reference = Identifier;

    // TODO -> Possible to merge cli & 1pux types?
    #[derive(Debug, Clone, Serialize, Deserialize, CommonFields)]
    #[serde(rename_all = "SCREAMING_SNAKE_CASE")]
    pub enum Vault {
        Personal {
            #[serde(flatten)]
            attrs: Attrs,
        },
        // TODO
        Shared {
            #[serde(flatten)]
            attrs: Attrs,
        },
        UserCreated {
            #[serde(flatten)]
            attrs: Attrs,
        },
    }

    #[cfg_attr(test, derive(Dummy))]
    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Attrs {
        #[serde(flatten)]
        pub reference: Identifier,

        /// TODO Docs
        pub content_version: usize,

        /// TODO Docs
        pub attribute_version: usize,

        /// Defines how many items are within this vault.
        pub items: usize,

        #[serde(flatten)]
        pub dated: Dated,
    }

    impl Display for Vault {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            self.attrs().reference.fmt(f)
        }
    }

    impl CliGetter<OnePasswordCore, Vault, [&'static str; 2]> for Vault {
        const ARGS: [&'static str; 2] = ["vault", "get"];
    }

    impl CliGetter<OnePasswordCore, Vec<Reference>, [&'static str; 2]> for Reference {
        const ARGS: [&'static str; 2] = ["vault", "list"];
    }

    impl Vault {
        pub async fn parse(account: &OnePasswordAccount, runtime: &Runtime) -> Result<Vec<Vault>> {
            let attrs = account.attrs();
            let vaults = &attrs.vaults;

            // let vaults = vaults
            //     .into_par_iter()
            //     .map(|r| {
            //         let envs = vec![];
            //         let args = vec![r.id()];
            //         Vault::get(config, account, &envs, &args)
            //     })
            //     .collect::<Vec<_>>();

            let envs = [];
            let mut finished_vaults = vec![];
            for vault in vaults {
                let args = [vault.id()];
                let vault = Vault::get(runtime, account, &envs, &args);

                match vault.await {
                    Ok(vault) => {
                        trace!("Retrieved vault: {}", vault.attrs().reference);
                        finished_vaults.push(vault);
                    }
                    Err(e) => {
                        warn!("Unable to get long form vault: {}", e);
                        continue;
                    }
                };
            }

            Ok(finished_vaults)
        }
    }

    impl From<Vault> for one_pux::vault::Attrs {
        fn from(val: Vault) -> Self {
            use one_pux::vault::{Attrs, Type};

            let vault = val.attrs();
            Attrs {
                uuid: vault.reference.id().to_owned(),
                desc: "".to_string(),   // TODO
                avatar: "".to_string(), // TODO
                name: vault.reference.named().to_owned(),
                vault_type: match val {
                    Vault::Personal { .. } => Type::P,
                    Vault::Shared { .. } => Type::E,
                    Vault::UserCreated { .. } => Type::U,
                },
            }
        }
    }
}

pub mod url {
    use super::super::one_pux;
    use serde::{Deserialize, Serialize};
    use tracing::instrument;

    #[cfg(test)]
    use fake::{faker::internet::en::DomainSuffix, Dummy};

    #[cfg_attr(test, derive(Dummy))]
    #[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
    pub struct Url {
        /// The user defined & editable label for the url.
        #[serde(default)]
        pub label: Option<String>,

        /// Whether this is the primary url.
        #[cfg_attr(test, dummy(faker = "false"))]
        #[serde(default)]
        pub primary: bool,

        /// The url itself.
        #[cfg_attr(test, dummy(faker = "DomainSuffix()"))]
        pub href: String,
    }

    pub trait UrlsExt {
        fn get_primary(&self) -> Option<&Url>;
    }

    impl UrlsExt for Vec<Url> {
        fn get_primary(&self) -> Option<&Url> {
            self.iter().find(|u| u.primary)
        }
    }

    impl From<Url> for one_pux::item::UrlObject {
        #[instrument(name = "Url -> UrlObject")]
        fn from(val: Url) -> Self {
            one_pux::item::UrlObject {
                url: val.href,
                label: val.label.unwrap_or_default(),
                mode: "default".to_string(), // Unable to get from CLI
            }
        }
    }

    #[cfg(test)]
    pub(crate) mod test {
        use super::*;
        use fake::{Fake, Faker};
        use rand::prelude::IteratorRandom;
        use serde_json::json;

        #[test]
        fn test_url() {
            let reference: Url = Faker.fake();
            let json = json!({
                "label": reference.label,
                "primary": reference.primary,
                "href": reference.href,
            })
            .to_string();

            let url = serde_json::from_str::<Url>(&json);
            assert!(url.is_ok(), "Url should deserialize correctly.");
            let url = url.unwrap();

            assert_eq!(url.href, reference.href);
            assert_eq!(url.label, reference.label);
            assert_eq!(url.primary, reference.primary);
        }

        #[test]
        fn test_empty_urls() {
            let urls: Vec<Url> = vec![];
            assert!(
                urls.get_primary().is_none(),
                "Should return None when there are no urls."
            );
        }

        #[test]
        fn test_no_primary() {
            let urls: Vec<Url> = fake::vec![super::Url; 3..10];
            assert!(
                urls.get_primary().is_none(),
                "Should return None when no primary url is present."
            );
        }

        #[test]
        fn test_has_primary() {
            let mut urls: Vec<Url> = fake::vec![super::Url; 3..10];
            let index = (0..urls.len()).choose(&mut rand::thread_rng()).unwrap();
            urls[index].primary = true;
            assert!(
                urls.get_primary().is_some(),
                "Should return the primary when one is present."
            );
        }

        #[test]
        fn test_primary_matches() {
            let mut urls: Vec<Url> = fake::vec![super::Url; 3..10];
            let index = (0..urls.len()).choose(&mut rand::thread_rng()).unwrap();
            urls[index].primary = true;
            let primary = urls.get_primary().unwrap();
            assert_eq!(
                primary, &urls[index],
                "The returned url should have been the set via random index."
            );
        }
    }
}

pub mod field {
    use super::super::one_pux;
    use crate::sources::op::cli::reference_of;
    use macros::CommonFields;
    use serde::de::{SeqAccess, Visitor};
    use serde::{Deserialize, Deserializer, Serialize};
    use std::fmt;
    use std::str::FromStr;
    use tracing::debug;

    #[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
    #[serde(rename_all = "SCREAMING_SNAKE_CASE")]
    pub enum Purpose {
        Username,
        Password,
        Notes,
    }

    #[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
    #[serde(rename_all = "SCREAMING_SNAKE_CASE")]
    pub enum PasswordStrength {
        /// Represents 0-30
        Terrible,
        /// Represents 31-45
        Weak,
        /// Represents 46-55
        Fair,
        /// Represents 56-60
        Good,
        /// From 61-75
        VeryGood,
        /// From 76-80
        Excellent,
        /// From 81-100
        Fantastic,
    }

    #[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
    #[serde(rename_all = "camelCase")]
    pub struct PasswordDetails {
        // TODO -> Might be deprecated due to root root level entropy field
        #[serde(default)]
        pub entropy: usize,
        #[serde(default, skip_serializing_if = "one_pux::not_default")]
        pub generated: bool,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub strength: Option<PasswordStrength>,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        pub history: Vec<String>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
    pub struct Attrs {
        #[serde(flatten)]
        pub identifier: super::identifier::Identifier,

        /// If present references the additional section the field belongs to.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub section: Option<super::section::Section>,

        /// Defines that this field is a special field with a specific purpose.
        #[serde(default)]
        pub purpose: Option<Purpose>,

        /// The value/data of the field if present.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub value: Option<String>,

        /// Points to the fields location in the vault
        /// ## Format: `op://{vault}/{field type}/({section}/|empty){label}`
        pub reference: String,
    }

    impl Attrs {
        #[allow(unused)]
        pub(crate) fn new(
            parent_attrs: &super::item::Attrs,
            identifier: super::identifier::Identifier,
            section: Option<super::section::Section>,
            purpose: Option<Purpose>,
            value: Option<String>,
        ) -> Self {
            let reference = reference_of(parent_attrs, section.as_ref(), &identifier);

            Self {
                identifier,
                section,
                purpose,
                value,
                reference,
            }
        }
    }

    #[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize, CommonFields)]
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

            /// The currently active OTP code for the field.
            #[serde(default, skip_serializing_if = "Option::is_none")]
            totp: Option<String>,
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
        /// Defines a reference to another item;
        /// This item can be within the same vault or another vault.
        Reference {
            #[serde(flatten)]
            attrs: Attrs,
        },
        // TODO: Investigate further
        /// Seems to be a fallback for the cli to handle fields it hasn't updated to support yet.
        /// I've only seen this happen with the sso field type.
        Unknown {
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

            attrs.section.is_none() && attrs.purpose.as_ref().is_some_and(|p| Self::LOGIN_PURPOSES.contains(p))
        }

        pub fn is_notes_field(&self) -> bool {
            match self {
                Field::String { attrs, .. } => {
                    attrs.section.is_none() && attrs.purpose.as_ref().is_some_and(|p| p == &Purpose::Notes)
                }
                _ => false,
            }
        }
    }

    impl From<PasswordStrength> for usize {
        /// Converts a password strength to a percentage.
        /// Since we can't extract the exact number from the cli,
        /// we'll just use the lower bound of the range.
        fn from(val: PasswordStrength) -> Self {
            match val {
                PasswordStrength::Terrible => 0,
                PasswordStrength::Weak => 31,
                PasswordStrength::Fair => 46,
                PasswordStrength::Good => 56,
                PasswordStrength::VeryGood => 61,
                PasswordStrength::Excellent => 76,
                PasswordStrength::Fantastic => 81,
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
            let attrs = val.attrs().clone();
            one_pux::item::Field {
                id: match &attrs.identifier.named() {
                    l if l == &attrs.identifier.named() => "",
                    l => l,
                }
                .to_string(), // TODO :: Clear if same as name i think, needs checking
                name: attrs.identifier.named().to_owned(),
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
            let mut field = match val {
                Field::String { ref attrs } => one_pux::section::Field {
                    value: one_pux::section::Value::String(attrs.value.clone().unwrap_or_default()),
                    ..Default::default()
                },
                Field::Concealed { ref attrs, .. } => one_pux::section::Field {
                    value: one_pux::section::Value::Concealed(attrs.value.clone().unwrap_or_default()),
                    ..Default::default()
                },
                Field::Otp { ref attrs, .. } => one_pux::section::Field {
                    value: one_pux::section::Value::TOTP(attrs.value.clone().unwrap_or_default()),
                    input_traits: one_pux::section::InputTraits {
                        correction: one_pux::section::Correction::No,
                        capitalization: one_pux::section::Capitalization::None,
                        ..Default::default()
                    },
                    ..Default::default()
                },
                Field::CreditCardNumber { ref attrs } => one_pux::section::Field {
                    value: one_pux::section::Value::CreditCardNumber(attrs.value.clone().unwrap_or_default()),
                    guarded: true,
                    clipboard_filter: Some("0123456789".to_string()),
                    input_traits: one_pux::section::InputTraits {
                        keyboard: one_pux::section::Keyboard::NumberPad,
                        ..Default::default()
                    },
                    ..Default::default()
                },
                Field::Menu { ref attrs } => one_pux::section::Field {
                    value: one_pux::section::Value::Menu(attrs.value.clone().unwrap_or_default()),
                    ..Default::default()
                },
                Field::Date { ref attrs } => one_pux::section::Field {
                    value: one_pux::section::Value::Date(attrs.value.clone().map(|v| usize::from_str(&v).unwrap())),
                    ..Default::default()
                },
                Field::MonthYear { ref attrs } => one_pux::section::Field {
                    value: one_pux::section::Value::MonthYear(
                        attrs.value.clone().map(|v| usize::from_str(&v).unwrap()),
                    ),
                    ..Default::default()
                },
                Field::Url { ref attrs } => one_pux::section::Field {
                    value: one_pux::section::Value::Url(attrs.value.clone().unwrap_or_default()),
                    input_traits: one_pux::section::InputTraits {
                        keyboard: one_pux::section::Keyboard::URL,
                        ..Default::default()
                    },
                    ..Default::default()
                },
                Field::Phone { ref attrs } => one_pux::section::Field {
                    value: one_pux::section::Value::Phone(attrs.value.clone().unwrap_or_default()),
                    input_traits: one_pux::section::InputTraits {
                        keyboard: one_pux::section::Keyboard::NamePhonePad,
                        ..Default::default()
                    },
                    ..Default::default()
                },
                Field::Address { ref attrs } => {
                    let value = attrs.value.clone().unwrap_or_default();
                    let mut split = value.split(", ").collect::<Vec<&str>>();
                    let mut next = || -> String {
                        if split.is_empty() {
                            return String::new();
                        }

                        let value = split.remove(0);
                        if value == "<nil>" {
                            return String::new();
                        }

                        value.to_string()
                    };

                    one_pux::section::Field {
                        value: one_pux::section::Value::Address {
                            street: next(),
                            city: next(),
                            state: next(),
                            zip: next(),
                            country: next(),
                        },
                        ..Default::default()
                    }
                }
                Field::Email { ref attrs } => one_pux::section::Field {
                    value: one_pux::section::Value::Email {
                        email_address: attrs.value.clone().unwrap_or_default(),
                        provider: None, // TODO
                    },
                    guarded: true,
                    input_traits: one_pux::section::InputTraits {
                        keyboard: one_pux::section::Keyboard::EmailAddress,
                        ..Default::default()
                    },
                    ..Default::default()
                },
                Field::SshKey { ref attrs } => one_pux::section::Field {
                    value: one_pux::section::Value::SshKey {
                        private_key: attrs.value.clone().unwrap_or_default(),
                        metadata: one_pux::section::SshKeyMetadata {
                            private_key: attrs.value.clone().unwrap_or_default(),
                            // These values are all their own individual fields
                            public_key: "".to_string(),  // TODO
                            fingerprint: "".to_string(), // TODO
                            key_type: "".to_string(),    // TODO
                        },
                    },
                    guarded: true,
                    ..Default::default()
                },
                Field::Reference { ref attrs, .. } => one_pux::section::Field {
                    value: one_pux::section::Value::Reference(attrs.value.clone().unwrap_or_default()),
                    ..Default::default()
                },
                Field::Unknown { ref attrs } => one_pux::section::Field {
                    value: one_pux::section::Value::String(attrs.value.clone().unwrap_or_default()),
                    ..Default::default()
                },
            };

            let attrs = val.attrs().clone();
            field.title = attrs.identifier.named().to_owned();
            field.id = attrs.identifier.id().to_owned();
            // TODO -> This is a bit hacky but it works for some items, if the field doesn't have a nl we can't know that it should be multiline
            field.multiline = attrs.value.as_ref().is_some_and(|v| v.contains('\n'));
            field
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
                debug!("Visiting seq with hint of {}", seq.size_hint().unwrap_or(usize::MAX));
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

    #[cfg(test)]
    pub(crate) mod test {
        use super::*;
        use crate::sources::op::cli::{identifier, item};
        use fake::faker::boolean::en::Boolean;
        use fake::faker::internet::en::*;
        use fake::faker::lorem::en::*;
        use fake::Fake;

        /// Creates a randomised login and password field.
        pub(crate) fn random_login(parent_attrs: &item::Attrs) -> [Field; 2] {
            [
                Field::String {
                    attrs: Attrs::new(
                        parent_attrs,
                        identifier::Identifier {
                            id: identifier::Id("username".into()),
                            unique_name: identifier::UniqueName::Label("username".into()),
                        },
                        None,
                        Some(Purpose::Username),
                        Some(Username().fake()),
                    ),
                },
                Field::Concealed {
                    attrs: Attrs::new(
                        parent_attrs,
                        identifier::Identifier {
                            id: identifier::Id("password".into()),
                            unique_name: identifier::UniqueName::Label("password".into()),
                        },
                        None,
                        Some(Purpose::Password),
                        Some(Password(4..128).fake()),
                    ),
                    password_details: Some(PasswordDetails {
                        entropy: (0..100).fake(),
                        strength: Some(PasswordStrength::Weak), // TODO
                        history: fake::vec![String as Password(4..128); 0..5],
                        generated: Boolean(50).fake(),
                    }),
                },
            ]
        }

        pub(crate) fn random_notes(parent_attrs: &item::Attrs) -> Option<Field> {
            // Randomly decide if we should have a notes field
            if Boolean(20).fake() {
                return None;
            }

            Some(Field::String {
                attrs: Attrs::new(
                    parent_attrs,
                    identifier::Identifier {
                        id: identifier::Id("notesPlain".into()),
                        unique_name: identifier::UniqueName::Label("notesPlain".into()),
                    },
                    None,
                    Some(Purpose::Notes),
                    Some(Sentence(1..3).fake()),
                ),
            })
        }
    }
}

pub mod section {
    use super::super::one_pux;

    pub type Section = super::identifier::Identifier;

    impl From<Section> for one_pux::item::AdditionalSection {
        fn from(value: Section) -> Self {
            Self {
                title: value.named().to_owned(),
                name: value.id().to_owned(),
                ..Default::default()
            }
        }
    }

    #[cfg(test)]
    mod test {
        use super::*;
        use fake::{Fake, Faker};
        use one_pux::item::AdditionalSection;

        #[test]
        fn test_from() {
            let section: Section = Faker.fake();
            let one_pux_section: AdditionalSection = section.clone().into();

            assert_eq!(one_pux_section.title, section.named());
            assert_eq!(one_pux_section.name, section.id());
        }
    }
}

/// Creates a reference string for 1password,
/// This is used as a short url to the item.
fn reference_of(item: &item::Attrs, section: Option<&section::Section>, identifier: &identifier::Identifier) -> String {
    format!(
        "op:://{vault}/{item}/{inner_reference}",
        vault = item.vault.named(),
        item = item.identifier.id(),
        inner_reference = match &section {
            None => identifier.id().to_string(),
            Some(s) => format!("{}/{}", s.id(), identifier.id()),
        }
    )
}
