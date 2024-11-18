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
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::any::Any;
use std::str::FromStr;

pub mod cdata {
    use serde::{Deserialize, Serialize};
    use std::cell::{LazyCell, OnceCell};
    use std::error::Error;
    use std::ops::{Deref, DerefMut};
    use std::str::FromStr;
    use std::sync::LazyLock;

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    pub struct CData<T: FromStr> {
        #[serde(rename(deserialize = "$text"))]
        text: String,

        #[serde(skip)]
        phantom: std::marker::PhantomData<T>,
    }

    impl<T: FromStr> CData<T> {
        pub fn get(&self) -> anyhow::Result<T> {
            self.text.trim().parse().map_err(|_| {
                anyhow::anyhow!(
                    "Unable to parse the string [{}] into type [{}]",
                    self.text,
                    std::any::type_name::<T>(),
                )
            })
        }
    }
}

pub mod xml {
    use serde::{Deserialize, Serialize};

    #[derive(PartialEq, Debug, Serialize, Deserialize)]
    pub struct Items<I> {
        #[serde(rename(deserialize = "$value"))]
        pub items: I,
    }

    #[derive(PartialEq, Debug, Serialize, Deserialize)]
    #[serde(rename = "result")]
    pub struct XMLResult<I: Sized> {
        #[serde(rename(deserialize = "@created"))]
        pub created: String,
        #[serde(rename(deserialize = "@host"))]
        pub host: String,
        #[serde(rename(deserialize = "@status"))]
        pub status: String,
        #[serde(rename(deserialize = "$value"))]
        pub items: Items<I>,
    }
}

pub mod client {
    use crate::endpoints::nable::structs::cdata::CData;
    use chrono::NaiveDate;
    use serde::{Deserialize, Serialize};

    pub type Clients = Vec<Client>;

    /// Derived from https://documentation.n-able.com/remote-management/userguide/Content/listing_clients_.htm
    #[derive(PartialEq, Debug, Serialize, Deserialize)]
    #[serde(rename = "client")]
    pub struct Client {
        #[serde(rename = "name")]
        pub identity_name: CData<String>,
        #[serde(rename = "clientid")]
        pub identity_id: usize,

        #[serde(rename = "dashboard_username")]
        pub client_login_username: Option<CData<String>>,

        pub view_dashboard: bool,
        pub view_wkstsn_assets: bool,

        #[serde(rename = "timezone")]
        pub meta_timezone: Option<CData<String>>,
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
        use crate::endpoints::nable::structs::xml::XMLResult;

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
    use crate::endpoints::nable::structs::cdata::CData;
    use chrono::NaiveDate;
    use serde::{Deserialize, Serialize};

    pub type Sites = Vec<Site>;

    #[derive(Debug, Serialize, Deserialize)]
    pub struct Site {
        #[serde(rename = "name")]
        pub identity_name: String,

        #[serde(rename = "siteid")]
        pub identity_id: usize,

        pub primary_router: Option<CData<String>>,
        pub secondary_router: Option<CData<String>>,

        pub connection_ok: bool,

        #[serde(rename = "creation_date", deserialize_with = "crate::deserialise_date_opt")]
        pub meta_creation: Option<NaiveDate>,
    }
}

pub mod template {
    use crate::endpoints::nable::structs::cdata::CData;
    use serde::{Deserialize, Serialize};

    pub type Templates = Vec<Template>;

    #[derive(Debug, Serialize, Deserialize)]
    #[serde(rename = "installation_template")]
    pub struct Template {
        #[serde(rename = "name")]
        pub identity_name: Option<CData<String>>,

        #[serde(rename = "templateid")]
        pub identity_id: usize,
    }
}

pub mod device {
    use crate::endpoints::nable::structs::cdata::CData;
    use macros::CommonFields;
    use serde::{Deserialize, Serialize};

    pub type Servers = Vec<Device>;
    pub type Workstations = Vec<Device>;

    #[derive(Debug, Serialize, Deserialize)]
    pub struct BaseDevice {
        #[serde(rename(deserialize = "name"))]
        pub identity_name: CData<String>,

        #[serde(alias = "workstationid", alias = "serverid")]
        pub identity_id: CData<usize>,

        #[serde(rename = "guid")]
        pub identity_guid: Option<CData<String>>,

        #[serde(rename = "description")]
        pub meta_description: Option<CData<String>>,

        #[serde(rename = "agent_version")]
        pub meta_agent_version: Option<CData<String>>,

        #[serde(rename = "agent_mode")]
        pub meta_agent_mode: CData<usize>,

        // #[serde(rename = "install_date")]
        // pub meta_install_date: NaiveDate,
        #[serde(rename = "last_boot_time")]
        pub meta_boot_time: Option<CData<String>>,

        #[serde(rename = "online")]
        // #[serde_as(as = "BoolFromInt")]
        pub status_online: CData<bool>,
        // #[serde(rename = "active_247")]
        // #[serde_as(as = "BoolFromInt")]
        // pub checks_active: bool,
        //
        // #[serde(rename = "check_interval_247")]
        // pub checks_interval: usize,
        //
        // #[serde(rename = "status_247")]
        // pub checks_status: usize,
        //
        // // #[serde(rename = "local_date_247")]
        // // pub checks_date: NaiveDate,
        //
        // // #[serde(rename = "local_time_247")]
        // // pub checks_time: NaiveTime,
        //
        // // #[serde(rename = "utc_time_247")]
        // // pub checks_time_utc: Option<NaiveDateTime>,
        // #[serde(rename = "dsc_active")]
        // #[serde_as(as = "BoolFromInt")]
        // pub daily_active: bool,
        //
        // #[serde(rename = "dsc_hour")]
        // pub daily_hour: usize,
        //
        // #[serde(rename = "dsc_status")]
        // pub daily_status: usize,
        //
        // // #[serde(rename = "dsc_local_date")]
        // // pub daily_date: NaiveDate,
        //
        // // #[serde(rename = "dsc_local_time")]
        // // pub daily_time: NaiveTime,
        //
        // // #[serde(rename = "dsc_utc_time")]
        // // pub daily_time_utc: Option<NaiveDateTime>,
        // #[serde(rename = "tz_bias")]
        // pub timezone_bias: Option<isize>,
        // #[serde(rename = "tz_dst_bias")]
        // pub timezone_bias_daylight_saving: Option<isize>,
        // #[serde(rename = "tz_std_bias")]
        // pub timezone_bias_standard_bias: Option<isize>,
        // #[serde(rename = "tz_mode")]
        // pub timezone_bias_mode: Option<usize>,
        // #[serde(rename = "tz_dst_date")]
        // pub timezone_date_daylight_saving: Option<String>,
        // #[serde(rename = "tz_std_date")]
        // pub timezone_date_standard: Option<String>,
        // #[serde(rename = "atz_dst_date")]
        // pub timezone_date_daylight_saving_alt: Option<String>,
        //
        // // pub utc_apt: Option<NaiveDateTime>,
        // pub utc_offset: Option<isize>,
        //
        // #[serde(rename = "assetid")]
        // pub asset_id: Option<usize>,
        // pub wins_name: Option<CData>,
        // pub user: Option<CData>,
        // pub domain: Option<CData>,
        // pub role: Option<String>,
        // pub chassis_type: Option<isize>,
        // pub manufacturer: Option<CData>,
        // pub model: Option<CData>,
        // pub device_serial: Option<CData>,
        // pub ip: Option<CData>,
        // pub external_ip: Option<CData>,
        // pub mac1: Option<CData>,
        // pub mac2: Option<CData>,
        // pub mac3: Option<CData>,
        // pub processor_count: Option<usize>,
        // pub total_memory: Option<usize>,
        // pub os_type: usize,
        // pub os: Option<CData>,
        // pub service_pack: Option<f64>,
        // pub os_serial_number: Option<CData>,
        // pub os_product_key: Option<CData>,
        // // pub last_scan_time: Option<DateTime<Utc>>,
    }

    #[derive(Debug, Serialize, Deserialize, CommonFields)]
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

            remote_address: Option<CData<String>>,

            remote_port: Option<CData<String>>,

            remote_username: Option<CData<String>>,

            remote_domain: Option<CData<String>>,
        },
        Workstation {
            #[serde(flatten)]
            base: BaseDevice,
        },
    }
}
