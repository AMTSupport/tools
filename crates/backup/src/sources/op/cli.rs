pub mod identifier {
    use std::fmt::{Display, Formatter};

    #[derive(Default, Debug, Clone, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
    pub struct Identifier {
        // TODO :: Seems like its an enum with the rename_all of camelCase
        /// The internal identifier for this entity.
        #[serde(default, skip_serializing_if = "String::is_empty")]
        pub id: String,

        /// The user facing & modifiable name for this entity.
        #[serde(default, skip_serializing_if = "String::is_empty")]
        pub label: String,
    }

    impl Display for Identifier {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            write!(f, "{label}-{id}", label = &self.label, id = &self.id)
        }
    }
}

pub mod state {
    use serde::{Deserialize, Serialize};
    use std::fmt::{Display, Formatter};

    // TODO -> Are there any other states?
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

    #[derive(Default, Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
    pub struct Dated {
        pub created_at: DateTime<Utc>,
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
    use async_trait::async_trait;
    use chrono::{DateTime, FixedOffset, Utc};

    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Attrs {
        /// The internal user identifier.
        pub id: String,

        /// The users name.
        pub name: String,

        /// The users email address.
        pub email: String,

        /// The account state of the user.
        pub state: super::state::State,

        /// The date information for the user.
        #[serde(flatten)]
        pub dated: super::dated::Dated,

        /// When this user was last authenticated with the cli.
        pub last_auth_at: DateTime<Utc>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
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

    #[async_trait]
    impl CliGetter<OnePasswordCore, User, [&'static str; 3]> for User {
        const ARGS: [&'static str; 3] = ["user", "get", "--me"];
    }

    // TODO -> macro for this?
    impl User {
        const fn attrs(&self) -> &Attrs {
            match self {
                User::Member { attrs } => attrs,
                User::ServiceAccount { attrs } => attrs,
            }
        }

        pub fn id(&self) -> &str {
            &self.attrs().id
        }

        pub fn name(&self) -> &str {
            &self.attrs().name
        }

        pub fn email(&self) -> &str {
            &self.attrs().email
        }

        pub fn state(&self) -> &super::state::State {
            &self.attrs().state
        }

        pub fn dated(&self) -> &super::dated::Dated {
            &self.attrs().dated
        }

        pub fn last_auth_at(&self) -> &DateTime<Utc> {
            &self.attrs().last_auth_at
        }
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
            short: super::account::Short,
        },
        ServiceAccount {
            #[serde(rename = "URL")]
            url: String,
            #[serde(rename_all = "camelCase")]
            service_account_type: String, // TODO -> Enum?
        },
    }

    impl CliGetter<OnePasswordCore, WhoAmI, [&'static str; 1]> for WhoAmI {
        const ARGS: [&'static str; 1] = ["whoami"];
    }
}

pub mod account {
    use crate::config::runtime::RuntimeConfig;
    use crate::sources::getter::CliGetter;
    use crate::sources::op::core::OnePasswordCore;
    use crate::sources::op::one_pux;
    use async_trait::async_trait;
    use chrono::{DateTime, FixedOffset, Utc};
    use lib::anyhow::Result;
    use serde::{Deserialize, Serialize};

    /// This comes from the list of account gotten with `op list accounts`
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Short {
        pub url: String,
        pub email: String,
        pub user_uuid: String,
        pub account_uuid: String,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Attrs {
        /// The internal unique identifier for the account.
        pub id: String,

        /// The account owners name
        pub name: String,

        /// The 1password domain for this account
        /// ### This is only the subdomain and not the entire url.
        pub domain: String,

        /// The accounts current state.
        pub state: super::state::State,

        /// When this account was created.
        pub created_at: DateTime<Utc>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[serde(rename_all = "SCREAMING_SNAKE_CASE", tag = "type")]
    pub enum Account {
        Individual {
            #[serde(flatten)]
            attrs: Attrs,

            #[serde(default, skip)]
            short: Option<Short>,
        },
        Business {
            #[serde(flatten)]
            attrs: Attrs,

            #[serde(default, skip)]
            short: Option<Short>,
        },
    }

    impl CliGetter<OnePasswordCore, Vec<Short>, [&'static str; 2]> for Short {
        const ARGS: [&'static str; 2] = ["account", "list"];
    }

    impl CliGetter<OnePasswordCore, Account, [&'static str; 2]> for Account {
        const ARGS: [&'static str; 2] = ["account", "get"];
    }

    impl Account {
        fn get_attrs(&self) -> &Attrs {
            match self {
                Account::Individual { attrs, .. } => attrs,
                Account::Business { attrs, .. } => attrs,
            }
        }

        fn get_short(&self) -> Option<&Short> {
            match self {
                Account::Individual { short, .. } => short,
                Account::Business { short, .. } => short,
            }
            .as_ref()
        }
    }

    impl From<Account> for one_pux::account::Attrs {
        fn from(value: Account) -> one_pux::account::Attrs {
            let attrs = value.get_attrs().to_owned();
            let short = value.get_short();

            one_pux::account::Attrs {
                account_name: attrs.name.clone(),
                name: attrs.name,
                avatar: "".to_string(), // TODO -> Unsure if this is available // Is a filename from the archive
                email: short.map(|s| s.email).unwrap_or_default(),
                uuid: attrs.id,
                domain: format!("https://{}.1password.com/", value.get_attrs().domain),
            }
        }
    }

    #[cfg(test)]
    mod tests {

        #[test]
        fn test_short() {
            let json = r#"
{
  "url": "teamamt.1password.com",
  "email": "tangentmoons@gmail.com",
  "user_uuid": "MNNFSRC5WNAJLO4KMQ4EHVLQWA",
  "account_uuid": "OJMGKJBNAJHSRHRO6YY4DVWE6Y"
}
"#
            .trim();

            let short = serde_json::from_str::<super::Short>(json).unwrap();
            let serialised = serde_json::to_string_pretty(&short).unwrap();

            assert_eq!(json, serialised, "Serialisation should be the same");
        }

        #[test]
        fn test_account() {
            let json = r#"
{
  "type": "INDIVIDUAL",
  "id": "LAIQMSG1PWNMCA9LAS5KLSDURN",
  "name": "Test Name",
  "domain": "my",
  "state": "ARCHIVED",
  "created_at": "2023-02-05T05:07:29Z"
}
"#
            .trim();

            let account = serde_json::from_str::<super::Account>(json).unwrap();
            let serialised = serde_json::to_string_pretty(&account).unwrap();

            assert_eq!(json, serialised, "Serialisation should be the same");
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
    use indicatif::{MultiProgress, ParallelProgressIterator, ProgressBar};
    use lib::{anyhow, anyhow::Context, anyhow::Result};
    use rayon::prelude::*;
    use serde::{Deserialize, Serialize};
    use serde_json::from_slice;
    use std::fmt::{Display, Formatter};
    use std::process::Command;
    use tracing::{error, instrument, trace};

    #[derive(Debug, Clone, Serialize, Deserialize)]
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
        state: super::state::State,

        /// A reference to the vault which owns this item.
        vault: super::vault::Reference,

        /// The UUID of the User which last edited this item
        last_edited_by: String,

        #[flatten]
        dated: super::dated::Dated,

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
                    password_strength: details.strength.map(|s| s.into()).unwrap_or_default(),
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

    impl TryFrom<Item> for one_pux::item::Details {
        type Error = anyhow::Error; // TODO -> ThisError

        fn try_from(value: Item) -> Result<one_pux::item::Details> {
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
                            let into = password_details
                                .clone() // TODO -> Better error handling
                                .expect(&*format!("Get password details of {value}"))
                                .into();
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
                        sections.insert(0, one_pux::item::AdditionalSection::default())
                    }

                    sections
                        .first_mut()
                        .context("Get empty section for fields.")?
                        .fields
                        .push(field.into());
                    continue;
                }

                let section_ref = attrs.section.as_ref().with_context(|| {
                    format!(
                        "Get section for field {} of item {value}.",
                        attrs.identifier,
                    )
                })?;

                sections
                    .iter_mut()
                    .find(|s| s.name == section_ref.identifier.id)
                    .map(|s| s.fields.push(field.into()));
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
                password_history: password_history
                    .context("Unwrap password history")
                    .unwrap_or_default(),
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

        pub fn get_attrs(&self) -> &Attrs {
            match self {
                Item::Login { attrs, .. } => attrs,
                Item::CreditCard { attrs, .. } => attrs,
                Item::SecureNote { attrs, .. } => attrs,
                Item::Identity { attrs, .. } => attrs,
                Item::Password { attrs, .. } => attrs,
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

        fn raw(vault_id: &String, mut command: Command) -> Result<Vec<u8>> {
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

        fn raw_long(
            vault: &super::vault::Reference,
            item: Item,
            mut command: Command,
        ) -> Result<Vec<u8>> {
            command
                .args(["item", "get", &item.get_attrs().id])
                .args(["--vault", &vault.id, "--format=json"])
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

        #[instrument]
        pub fn parse(
            vault: super::vault::Vault,
            account: &&dyn AccountCommon,
            config: &RuntimeConfig,
            bars: (&ProgressBar, &MultiProgress),
        ) -> Result<Vec<Item>> {
            trace!("Requesting Items from {vault}");
            let bar = bars.1.insert_after(bars.0, lib::progress::spinner_with_count());

            bar.set_message(format!("Requesting items from `{vault}` vault...",));

            let items = Self::raw(&vault.reference.id, account.command(config))
                .and_then(|raw| from_slice::<Vec<Item>>(&raw).context("Deserialize items list"))?;

            bar.set_length(items.len() as u64);
            bar.set_message(format!("Requesting items details from `{vault}` vault...",));

            let items = items
                .into_par_iter()
                .progress_with(bar)
                .map(|item| Self::raw_long(&vault.reference, item, account.command(config)))
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
            write!(f, "{}-{}", self.get_attrs().id, self.get_attrs().title)
        }
    }
}

pub mod vault {
    use crate::config::runtime::RuntimeConfig;
    use crate::sources::getter::CliGetter;
    use crate::sources::op::account::AccountCommon;
    use crate::sources::op::core::OnePasswordCore;
    use crate::sources::op::one_pux;

    use chrono::{DateTime, FixedOffset};
    use lib::anyhow::{Context, Result};
    use serde::{Deserialize, Serialize};
    use serde_json::from_slice;

    use std::fmt::Display;
    use std::process::Command;
    use tracing::{error, trace};

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

    impl CliGetter<OnePasswordCore, Vec<Reference>, [&'static str; 2]> for Reference {
        const ARGS: [&'static str; 2] = ["vault", "list"];
    }

    impl Vault {
        fn raw(vault_id: &String, mut command: Command) -> Result<Vec<u8>> {
            command
                .args(["vault", "get", vault_id, "--format=json"])
                .output()
                .with_context(|| format!("Get vault {vault_id}."))
                .inspect(|o| {
                    if !o.status.success() {
                        error!("Failed to get item list from vault {vault_id}.");
                        error!("stdout: {}", String::from_utf8_lossy(&o.stdout));
                        error!("stderr: {}", String::from_utf8_lossy(&o.stderr));
                    }
                })
                .map(|o| o.stdout)
        }

        pub fn parse(account: &&dyn AccountCommon, config: &RuntimeConfig) -> Result<Vec<Vault>> {
            let vaults = account.vaults();

            let mut final_vaults = vec![];
            for vault in vaults {
                trace!("Requesting Vault {vault}",);
                let raw = Self::raw(&vault.id, account.command(config))?;
                let parsed = from_slice::<Vault>(raw.as_slice()).with_context(|| {
                    format!("Deserialise vault from {}", String::from_utf8_lossy(&raw))
                })?;
                trace!("Parsed Vault {parsed}");

                final_vaults.push(parsed);
            }

            Ok(final_vaults)
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
    use tracing::instrument;

    #[derive(Default, Debug, Clone, Serialize, Deserialize)]
    pub struct Url {
        /// The user defined & editable label for the url.
        #[serde(default)]
        pub label: Option<String>,

        /// Whether this is the primary url.
        #[serde(default)]
        pub primary: bool,

        /// The url itself.
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
}

pub mod field {
    use super::super::one_pux;

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
                Field::Reference { attrs, .. } => attrs,
                Field::Unknown { attrs, .. } => attrs,
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
            let attrs = val.get_attrs().clone();
            one_pux::item::Field {
                id: match &attrs.identifier.label {
                    l if l == &attrs.identifier.label => "",
                    l => l,
                }
                .to_string(), // TODO :: Clear if same as name i think, needs checking
                name: attrs.identifier.label,
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
                Field::String { attrs } => one_pux::section::Field {
                    value: one_pux::section::Value::String(attrs.value.unwrap_or_default()),
                    ..Default::default()
                },
                Field::Concealed { attrs, .. } => one_pux::section::Field {
                    value: one_pux::section::Value::Concealed(attrs.value.unwrap_or_default()),
                    ..Default::default()
                },
                Field::Otp { attrs, .. } => one_pux::section::Field {
                    value: one_pux::section::Value::TOTP(attrs.value.unwrap_or_default()),
                    input_traits: one_pux::section::InputTraits {
                        correction: one_pux::section::Correction::No,
                        capitalization: one_pux::section::Capitalization::None,
                        ..Default::default()
                    },
                    ..Default::default()
                },
                Field::CreditCardNumber { attrs } => one_pux::section::Field {
                    value: one_pux::section::Value::CreditCardNumber(
                        attrs.value.unwrap_or_default(),
                    ),
                    guarded: true,
                    clipboard_filter: Some("0123456789".to_string()),
                    input_traits: one_pux::section::InputTraits {
                        keyboard: one_pux::section::Keyboard::NumberPad,
                        ..Default::default()
                    },
                    ..Default::default()
                },
                Field::Menu { attrs } => one_pux::section::Field {
                    value: one_pux::section::Value::Menu(attrs.value.unwrap_or_default()),
                    ..Default::default()
                },
                Field::Date { attrs } => one_pux::section::Field {
                    value: one_pux::section::Value::Date(
                        attrs.value.map(|v| usize::from_str(&v).unwrap()),
                    ),
                    ..Default::default()
                },
                Field::MonthYear { attrs } => one_pux::section::Field {
                    value: one_pux::section::Value::MonthYear(
                        attrs.value.map(|v| usize::from_str(&v).unwrap()),
                    ),
                    ..Default::default()
                },
                Field::Url { attrs } => one_pux::section::Field {
                    value: one_pux::section::Value::Url(attrs.value.unwrap_or_default()),
                    input_traits: one_pux::section::InputTraits {
                        keyboard: one_pux::section::Keyboard::URL,
                        ..Default::default()
                    },
                    ..Default::default()
                },
                Field::Phone { attrs } => one_pux::section::Field {
                    value: one_pux::section::Value::Phone(attrs.value.unwrap_or_default()),
                    multiline,
                    input_traits: one_pux::section::InputTraits {
                        keyboard: one_pux::section::Keyboard::NamePhonePad,
                        ..Default::default()
                    },
                    ..Default::default()
                },
                Field::Address { attrs } => {
                    let value = attrs.value.unwrap_or_default();
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
                Field::Email { attrs } => one_pux::section::Field {
                    value: one_pux::section::Value::Email {
                        email_address: attrs.value.unwrap_or_default(),
                        provider: None, // TODO
                    },
                    guarded: true,
                    input_traits: one_pux::section::InputTraits {
                        keyboard: one_pux::section::Keyboard::EmailAddress,
                        ..Default::default()
                    },
                    ..Default::default()
                },
                Field::SshKey { attrs } => one_pux::section::Field {
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
                    ..Default::default()
                },
                Field::Reference { attrs, .. } => one_pux::section::Field {
                    value: one_pux::section::Value::Reference(attrs.value.unwrap_or_default()),
                    ..Default::default()
                },
                Field::Unknown { attrs } => one_pux::section::Field {
                    value: one_pux::section::Value::String(attrs.value.unwrap_or_default()),
                    ..Default::default()
                },
            };

            let attrs = val.get_attrs().clone();
            field.title = attrs.identifier.label;
            field.id = attrs.identifier.id;
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
        #[serde(flatten)]
        pub identifier: super::identifier::Identifier,
    }

    impl From<Section> for one_pux::item::AdditionalSection {
        fn from(value: Section) -> Self {
            Self {
                title: value.identifier.label,
                name: value.identifier.id,
                ..Default::default()
            }
        }
    }
}
