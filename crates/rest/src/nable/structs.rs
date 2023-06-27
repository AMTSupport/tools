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

pub mod xml {
    use serde::Deserialize;

    #[derive(PartialEq, Debug, Deserialize)]
    pub struct Items<I> {
        #[serde(rename = "$value")]
        pub clients: I,
    }

    #[derive(PartialEq, Debug, Deserialize)]
    #[serde(rename = "result")]
    pub struct XMLResult<I> {
        #[serde(rename = "@created")]
        pub created: String,
        #[serde(rename = "@host")]
        pub host: String,
        #[serde(rename = "@status")]
        pub status: String,
        #[serde(rename = "$value")]
        pub items: Items<I>,
    }
}

pub mod client {

    use crate::deserialise_date;
    use anyhow::Context;

    use chrono::NaiveDate;
    use quick_xml::{events::Event, Reader};
    use serde::{Deserialize, Deserializer};

    pub fn deserialise_raw<'de, D>(deserializer: D) -> Result<String, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let mut reader = Reader::from_str(&s);
        reader.trim_text(true);

        loop {
            match reader.read_event().context("Read event for raw deserialization").unwrap() {
                Event::Text(bytes) => {
                    return Ok(String::from_utf8_lossy(bytes.as_ref()).to_string())
                }
                _ => break,
            }
        }

        Err(serde::de::Error::custom(format!(
            "Failed to deserialise raw string: {:?}",
            s
        )))
    }

    pub fn deserialise_raw_opt<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let result = match String::deserialize(deserializer) {
            Ok(s) => match s {
                ref s if s.is_empty() || s == "none" => None,
                _ => Some(s),
            },
            Err(_) => None,
        };

        Ok(result)
    }

    pub type Clients = Vec<RMMClient>;

    #[derive(PartialEq, Debug, Deserialize)]
    #[serde(rename = "client")]
    pub struct RMMClient {
        #[serde(rename = "name", deserialize_with = "deserialise_raw")]
        pub identity_name: String,
        #[serde(rename = "clientid")]
        pub identity_id: usize,

        #[serde(
            rename = "dashboard_username",
            deserialize_with = "deserialise_raw_opt"
        )]
        pub client_login_username: Option<String>,
        pub view_dashboard: usize,
        pub view_wkstsn_assets: usize,
        #[serde(rename = "creation_date", deserialize_with = "deserialise_date")]
        pub meta_creation: NaiveDate,
        #[serde(rename = "device_count")]
        pub device_count_total: usize,
        #[serde(rename = "server_count")]
        pub device_count_servers: usize,
        #[serde(rename = "workstation_count")]
        pub device_count_workstations: usize,
        #[serde(rename = "mobile_device_count")]
        pub device_count_mobiles: usize,
    }

    // #[async_trait]
    // impl ClientFinder for RMMClient {
    //     async fn find(&self, client: Client) -> ClientGrouper {
    //         let hudu_company = client.get_companies();
    //     }
    // }
}
