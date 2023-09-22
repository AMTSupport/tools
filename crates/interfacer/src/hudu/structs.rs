/*
 * Copyright (C) 2023 James Draycott <me@racci.dev>
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, version 3.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */

pub mod company {
    use serde::Deserialize;

    pub type Companies = std::collections::HashMap<usize, Company>;

    #[derive(Clone, Debug, Deserialize)]
    #[serde(rename = "company")]
    pub struct Company {
        /// Unique identifier.
        pub id: usize,
        /// name.
        pub name: String,
        /// Unique slug.
        pub slug: String,
        /// Optional Nickname
        pub nickname: Option<String>,
        /// Parent company ID.
        pub parent_company_id: Option<usize>,
        /// Parent company name.
        pub parent_company_name: Option<String>,
        /// Optional notes.
        pub notes: Option<String>,
        /// Address line 1.
        pub address_line_1: Option<String>,
        /// Address line 2.
        pub address_line_2: Option<String>,
        /// City.
        pub city: Option<String>,
        /// State.
        pub state: Option<String>,
        /// Zip code.
        pub zip_code: Option<String>,
        /// Country.
        pub country: Option<String>,
        /// Phone number.
        pub phone_number: Option<String>,
        /// Fax number.
        pub fax_number: Option<String>,
        /// Website URL.
        pub website_url: Option<String>,
    }

    impl PartialEq for Company {
        fn eq(&self, other: &Self) -> bool {
            self.id == other.id
        }
    }

    impl Eq for Company {}
}

pub mod password {
    use crate::{hudu::API_ENDPOINT, Client, Url};
    use chrono::{DateTime, Utc};
    use serde::Deserialize;

    pub type Passwords = Vec<Password>;

    #[derive(Clone, Debug, Deserialize)]
    pub struct Password {
        #[serde(rename = "id")]
        pub identity_id: usize,
        #[serde(rename = "company_id")]
        pub identity_company_id: Option<usize>,
        #[serde(rename = "name")]
        pub identity_name: String,
        #[serde(rename = "slug")]
        pub identity_slug: String,

        #[serde(rename = "created_at")]
        pub meta_created_at: DateTime<Utc>,
        #[serde(rename = "updated_at")]
        pub meta_updated_at: DateTime<Utc>,

        #[serde(rename = "password_folder_id")]
        pub folder_id: Option<u8>,
        #[serde(rename = "password_folder_name")]
        pub folder_name: Option<String>,

        pub username: String,
        pub password: String,
        pub otp_secret: Option<String>,
    }

    impl Url<Client> for Password {
        fn link(&self, hudu: &Client) -> String {
            format!(
                "{url}/passwords/{slug}",
                url = hudu.base_url.strip_suffix(API_ENDPOINT).unwrap(),
                slug = &self.identity_slug
            )
        }
    }
}
