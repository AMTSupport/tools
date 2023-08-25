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

use anyhow::Context;
use quick_xml::events::Event;
use quick_xml::Reader;
use serde::{Deserialize, Deserializer};

pub mod xml {
    use serde::Deserialize;

    #[derive(PartialEq, Debug, Deserialize)]
    pub struct Items<I> {
        #[serde(rename = "$value")]
        pub items: I,
    }

    #[derive(PartialEq, Debug, Deserialize)]
    #[serde(rename = "result")]
    pub struct XMLResult<I: Sized> {
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
    use anyhow::Context;

    use chrono::{NaiveDate, TimeZone};
    use serde::{Deserialize, Deserializer};

    pub type Clients = Vec<Client>;

    /// Derived from https://documentation.n-able.com/remote-management/userguide/Content/listing_clients_.htm
    #[derive(PartialEq, Debug, Deserialize)]
    #[serde(rename = "client")]
    pub struct Client {
        #[serde(rename = "name", deserialize_with = "super::deserialise_cdata")]
        pub identity_name: String,
        #[serde(rename = "clientid")]
        pub identity_id: usize,

        #[serde(rename = "dashboard_username", deserialize_with = "super::deserialise_cdata_opt")]
        pub client_login_username: Option<String>,

        pub view_dashboard: bool,
        pub view_wkstsn_assets: bool,

        #[serde(rename = "timezone", deserialize_with = "super::deserialise_cdata_opt")]
        pub meta_timezone: Option<String>,
        #[serde(rename = "creation_date", deserialize_with = "crate::deserialise_date")]
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

    #[cfg(test)]
    mod test {
        use super::*;
        use crate::nable::structs::xml::XMLResult;

        const XML: &str = r#"
<?xml version="1.0" encoding="ISO-8859-1"?>
<result created="2023-08-11T16:34:10+10:00" host="www.system-monitor.com" status="OK">
    <items>
        <client>
            <name><![CDATA[cool client]]></name>
            <clientid>1289</clientid>
            <view_dashboard>0</view_dashboard>
            <view_wkstsn_assets>0</view_wkstsn_assets>
            <dashboard_username><![CDATA[none]]></dashboard_username>
            <timezone/>
            <creation_date>2011-08-08</creation_date>
            <server_count>0</server_count>
            <workstation_count>13</workstation_count>
            <mobile_device_count>0</mobile_device_count>
            <device_count>13</device_count>
        </client>
        <client>
            <name><![CDATA[the second super cool client]]></name>
            <clientid>68964</clientid>
            <view_dashboard>1</view_dashboard>
            <view_wkstsn_assets>1</view_wkstsn_assets>
            <dashboard_username><![CDATA[cool username]]></dashboard_username>
            <timezone><![CDATA[Australia/Sydney]]></timezone>
            <creation_date>2016-08-26</creation_date>
            <server_count>0</server_count>
            <workstation_count>8</workstation_count>
            <mobile_device_count>0</mobile_device_count>
            <device_count>8</device_count>
        </client>
    </items>
</result>
"#;

        #[test_log::test(test)]
        fn able_to_deserialise() {
            let result = quick_xml::de::from_str::<XMLResult<Clients>>(XML);
            assert!(result.is_ok(), "Failed to deserialise XML: {}", result.err().unwrap());
        }
    }

    // #[async_trait]
    // impl ClientFinder for RMMClient {
    //     async fn find(&self, client: Client) -> ClientGrouper {
    //         let hudu_company = client.get_companies();
    //     }
    // }
}

pub mod site {
    use chrono::NaiveDate;
    use serde::Deserialize;

    pub type Sites = Vec<Site>;

    #[derive(Debug, Deserialize)]
    pub struct Site {
        #[serde(rename = "name")]
        pub identity_name: String,

        #[serde(rename = "siteid")]
        pub identity_id: usize,

        #[serde(deserialize_with = "super::deserialise_cdata_opt")]
        pub primary_router: Option<String>,
        #[serde(deserialize_with = "super::deserialise_cdata_opt")]
        pub secondary_router: Option<String>,

        pub connection_ok: bool,

        #[serde(rename = "creation_date", deserialize_with = "crate::deserialise_date_opt")]
        pub meta_creation: Option<NaiveDate>,
    }
}

pub mod template {
    use serde::Deserialize;

    pub type Templates = Vec<Template>;

    #[derive(Debug, Deserialize)]
    #[serde(rename = "installation_template")]
    pub struct Template {
        #[serde(rename = "name", deserialize_with = "super::deserialise_cdata_opt")]
        pub identity_name: Option<String>,

        #[serde(rename = "templateid")]
        pub identity_id: usize,
    }
}

pub mod device {
    use chrono::{DateTime, FixedOffset, NaiveDate, NaiveDateTime, NaiveTime, Utc};
    use serde::Deserialize;

    pub type Servers = Vec<Device>;
    pub type Workstations = Vec<Device>;

    #[derive(Debug, Deserialize)]
    #[serde(rename = "server")]
    pub struct BaseDevice {
        #[serde(rename = "name", deserialize_with = "super::deserialise_cdata")]
        pub identity_name: String,

        #[serde(alias = "workstationid", alias = "serverid")]
        pub identity_id: usize,

        #[serde(rename = "guid", deserialize_with = "super::deserialise_cdata_opt")]
        pub identity_guid: Option<String>,

        #[serde(rename = "description", deserialize_with = "super::deserialise_cdata_opt")]
        pub meta_description: Option<String>,

        #[serde(rename = "agent_version", deserialize_with = "super::deserialise_cdata_opt")]
        pub meta_agent_version: Option<String>,

        #[serde(rename = "agent_mode")]
        pub meta_agent_mode: usize,

        #[serde(rename = "install_date")]
        pub meta_install_date: NaiveDate,

        #[serde(rename = "last_boot_time", deserialize_with = "super::deserialise_cdata_opt")]
        pub meta_boot_time: Option<String>,

        #[serde(rename = "online")]
        pub status_online: bool,

        #[serde(rename = "active_247")]
        pub checks_active: bool,

        #[serde(rename = "check_interval_247")]
        pub checks_interval: usize,

        #[serde(rename = "status_247")]
        pub checks_status: usize,

        #[serde(rename = "local_date_247")]
        pub checks_date: NaiveDate,

        #[serde(rename = "local_time_247")]
        pub checks_time: NaiveTime,

        // #[serde(rename = "utc_time_247")]
        // pub checks_time_utc: Option<NaiveDateTime>,
        #[serde(rename = "dsc_active")]
        pub daily_active: bool,

        #[serde(rename = "dsc_hour")]
        pub daily_hour: usize,

        #[serde(rename = "dsc_status")]
        pub daily_status: usize,

        #[serde(rename = "dsc_local_date")]
        pub daily_date: NaiveDate,

        #[serde(rename = "dsc_local_time")]
        pub daily_time: NaiveTime,

        // #[serde(rename = "dsc_utc_time")]
        // pub daily_time_utc: Option<NaiveDateTime>,

        #[serde(rename = "tz_bias")]
        pub timezone_bias: Option<isize>,
        #[serde(rename = "tz_dst_bias")]
        pub timezone_bias_daylight_saving: Option<isize>,
        #[serde(rename = "tz_std_bias")]
        pub timezone_bias_standard_bias: Option<isize>,
        #[serde(rename = "tz_mode")]
        pub timezone_bias_mode: Option<usize>,
        #[serde(rename = "tz_dst_date")]
        pub timezone_date_daylight_saving: Option<String>,
        #[serde(rename = "tz_std_date")]
        pub timezone_date_standard: Option<String>,
        #[serde(rename = "atz_dst_date")]
        pub timezone_date_daylight_saving_alt: Option<String>,

        // pub utc_apt: Option<NaiveDateTime>,
        pub utc_offset: Option<isize>,

        #[serde(rename = "assetid")]
        pub asset_id: Option<usize>,
        #[serde(deserialize_with = "super::deserialise_cdata_opt")]
        pub wins_name: Option<String>,
        #[serde(deserialize_with = "super::deserialise_cdata_opt")]
        pub user: Option<String>,
        #[serde(deserialize_with = "super::deserialise_cdata_opt")]
        pub domain: Option<String>,
        pub role: Option<String>,
        pub chassis_type: Option<isize>,
        #[serde(deserialize_with = "super::deserialise_cdata_opt")]
        pub manufacturer: Option<String>,
        #[serde(deserialize_with = "super::deserialise_cdata_opt")]
        pub model: Option<String>,
        #[serde(deserialize_with = "super::deserialise_cdata_opt")]
        pub device_serial: Option<String>,
        #[serde(deserialize_with = "super::deserialise_cdata_opt")]
        pub ip: Option<String>,
        #[serde(deserialize_with = "super::deserialise_cdata_opt")]
        pub external_ip: Option<String>,
        #[serde(deserialize_with = "super::deserialise_cdata_opt")]
        pub mac1: Option<String>,
        #[serde(deserialize_with = "super::deserialise_cdata_opt")]
        pub mac2: Option<String>,
        #[serde(deserialize_with = "super::deserialise_cdata_opt")]
        pub mac3: Option<String>,
        pub processor_count: Option<usize>,
        pub total_memory: Option<usize>,
        pub os_type: usize,
        #[serde(deserialize_with = "super::deserialise_cdata_opt")]
        pub os: Option<String>,
        pub service_pack: Option<f64>,
        #[serde(deserialize_with = "super::deserialise_cdata_opt")]
        pub os_serial_number: Option<String>,
        #[serde(deserialize_with = "super::deserialise_cdata_opt")]
        pub os_product_key: Option<String>,
        // pub last_scan_time: Option<DateTime<Utc>>,
    }

    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "lowercase")]
    pub enum Device {
        Server {
            #[serde(flatten)]
            base: BaseDevice,

            #[serde(rename = "miss_factor_247")]
            checks_missed: f64,

            #[serde(rename = "missed_247")]
            checks_overdue: bool,

            #[serde(rename = "email_overdue_alert")]
            alert_email: bool,

            #[serde(rename = "sms_overdue_alert")]
            alert_sms: bool,

            remote_connection_type: usize,

            #[serde(deserialize_with = "super::deserialise_cdata_opt")]
            remote_address: Option<String>,

            #[serde(deserialize_with = "super::deserialise_cdata_opt")]
            remote_port: Option<String>,

            #[serde(deserialize_with = "super::deserialise_cdata_opt")]
            remote_username: Option<String>,

            #[serde(deserialize_with = "super::deserialise_cdata_opt")]
            remote_domain: Option<String>,
        },
        Workstation(BaseDevice),
    }
}

fn deserialise_cdata<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de> + From<String>,
{
    let raw = String::deserialize(deserializer)?;
    let mut reader = Reader::from_str(&raw);
    reader.trim_text(true);

    loop {
        match reader.read_event().context("Read event for raw deserialization").unwrap() {
            Event::Text(bytes) => {
                let str = String::from_utf8_lossy(bytes.as_ref());
                return T::try_from(str.to_string()).map_err(serde::de::Error::custom);
            }
            _ => break,
        }
    }

    Err(serde::de::Error::custom(format!(
        "Failed to deserialise raw string [{:?}] into type [{}]",
        raw,
        std::any::type_name::<T>()
    )))
}

fn deserialise_cdata_opt<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de> + From<String>,
{
    return match String::deserialize(deserializer) {
        Ok(s) => match s {
            ref s if s.is_empty() || s == "none" => Ok(None),
            _ => Ok(Some(T::try_from(s).map_err(serde::de::Error::custom)?)),
        },
        Err(_) => Ok(None),
    };
}
