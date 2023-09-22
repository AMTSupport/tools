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

use crate::endpoints::endpoint::Endpoint;
use crate::endpoints::nable::structs::client::Clients;
use crate::endpoints::nable::structs::device::{Servers, Workstations};
use crate::endpoints::nable::structs::site::Sites;
use crate::endpoints::nable::structs::template::Templates;
use crate::endpoints::nable::structs::xml::XMLResult;
use crate::endpoints::nable::NSightApiKey;
use crate::rest;
use clap::{Parser, Subcommand};
use http_cache_reqwest::HttpCacheOptions;
use reqwest_middleware::ClientWithMiddleware;
use serde::Serialize;
use tracing::instrument;

// Endpoints not implemented:
// Get Client Sites (by client id or client name or just all sites)
// Get default Site Monitoring template
// Apply Site Monitoring template to Site

#[derive(Debug)]
pub struct NSightEndpoint {
    pub host: HostInfo,
    client: ClientWithMiddleware,
}

#[derive(Debug, Subcommand)]
pub enum Request {
    /// Gets a list of all clients.
    GetClients,

    /// Gets a list of the sites within the parent `clientid`.
    GetSites { clientid: String },

    /// Gets a list of all templates.
    GetTemplates,

    /// Gets a list of all servers within the parent `siteid`.
    GetServers { siteid: String },

    /// Gets a list of all workstations within the parent `siteid`.
    GetWorkstations { siteid: String },

    /// Gets a list of the unique checks for the `deviceid`.
    GetChecks { deviceid: String },

    /// Gets a list of all checks that are failing.
    /// If `clientid` is provided, only checks for that client will be returned.
    GetChecksFailing { clientid: Option<String> },

    /// Runs the check with the `checkid`.
    RunCheck { checkid: String },
}

#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum Response {
    Clients(XMLResult<Clients>),
    Sites(XMLResult<Sites>),
    Templates(XMLResult<Templates>),
    Servers(XMLResult<Servers>),
    Workstations(XMLResult<Workstations>),
    // Checks(XMLResult<()>),
}

#[derive(Debug, Parser)]
pub struct HostInfo {
    pub endpoint: String,
    pub api_key: NSightApiKey,
}

impl Endpoint for NSightEndpoint {
    type Args = HostInfo;
    type Request = Request;
    type Response = Response;

    // TODO :: validate api key
    #[instrument(level = "TRACE")]
    fn new(args: Self::Args) -> Self {
        use http_cache_reqwest::{Cache, CacheMode, HttpCache, MokaManager};
        use reqwest::Client;
        use reqwest_middleware::ClientBuilder;

        let builder = Client::builder().deflate(true);
        let client = ClientBuilder::new(builder.build().unwrap())
            .with(Cache(HttpCache {
                mode: CacheMode::ForceCache,
                manager: MokaManager::default(),
                options: HttpCacheOptions::default(),
            }))
            .build();

        Self { host: args, client }
    }

    #[instrument(level = "TRACE")]
    async fn handle(&self, event: Self::Request) -> anyhow::Result<Self::Response> {
        use Request::*;

        match event {
            GetClients => list_clients(&self.client, &self.host).await.map(Response::Clients),
            GetSites { clientid } => list_sites(&self.client, &self.host, &clientid).await.map(Response::Sites),
            GetTemplates => list_templates(&self.client, &self.host).await.map(Response::Templates),
            GetServers { siteid } => list_servers(&self.client, &self.host, &siteid).await.map(Response::Servers),
            GetWorkstations { siteid } => {
                list_workstations(&self.client, &self.host, &siteid).await.map(Response::Workstations)
            }
            GetChecks { deviceid } => {
                todo!()
                // get_checks(&self.client, &self.host, &deviceid).await.map(Response::Checks)
            }
            GetChecksFailing { clientid } => {
                todo!()
                // get_checks_failing(&self.client, &self.host, clientid.as_deref()).await.map(Response::Checks)
            }
            RunCheck { checkid } => {
                todo!()
            }
        }
    }
}

rest!(list_clients => Clients);
rest!(list_sites & [clientid] => Sites);
rest!(list_templates => Templates);

rest!(list_servers & [siteid] => Servers);
rest!(list_workstations & [siteid] => Workstations);

// rest!(list_checks & [deviceid] => Checks);
// rest!(list_checks_failing & (clientid) => Checks);

mod macros {
    #[macro_export]
    macro_rules! rest {
        ($service:ident => $ret:ty) => (rest!(@parse $service, $ret;););
        ($service:ident & [$($param:ident),*] & ($($opt_param:ident),*) => $ret:ty) => (rest!(@parse $service, $ret; $($param: str),* $($opt_param: Option<&str>),*););
        ($service:ident & ($($opt_param:ident),*) & [$($param:ident),*] => $ret:ty) => (rest!($service & [$($param: $param_type),*] & ($($opt_param),*) => $ret););
        ($service:ident & [$($param:ident),*] => $ret:ty) => (rest!($service & [$($param),*] & () => $ret););
        ($service:ident & ($($opt_param:ident),*) => $ret:ty) => (rest!($service & [] & ($($opt_param),*) => $ret););
        // (@parse $service:ident, $ret:ty; )
        // (@as_param $tts:tt) => (rest!(@as_param $tts););
        // (@as_param $tts:tt, $($param:tt)*) => (rest!(@as_param $($param)*));
        // (@param $param:expr) => ($param: &str);
        // (@param $param:ident: $param_type:ty) => ($param:ident: $param_type);
        // (@param $param:ident: $param_type:ty, $($param2:tt)*) => ($param:ident: $param_type, rest!(@param $($param2)*););
        // (@parse $service:ident, $ret:ty, $($param:tt)*, $($tail:tt)) => (rest!(@parse $service, $ret, {$($param)*},););

        // (@parse $service:ident, $ret:ty, {$($params:expr)*}, $(,)*) => {
        //
        // }
        // ($service:ident & [$($param:expr),*] => $ret:ty) => (rest!(@parse $service, rest!(@param $($param),*), $ret););

        // (@param $param:expr) => ($param: &str);
        // (@param $param:ident: $param_type:ty) => ($param:ident: $param_type);
        // Parse the $param:tt into a $param:ident and $param_type:ty
        // The expr can take the forms of:
        // 1. $param:ident
        // 2. $param:ident: $param_type:ty
        // This can be a single or multiple parameters
        // Implement this in a recursive macro where the params are parsed one by one.

        // (@param $param:ident) => ($param: &str);
        // (@param $param:ident, $param_type:ty) => ($param: $param_type);
        (@parse $service:ident, $ret:ty; $($param:ident: $param_type:ty),* $(,)?) => {
            #[tracing::instrument(skip(client))]
            pub async fn $service(
                client: &reqwest_middleware::ClientWithMiddleware,
                host_info: &crate::endpoints::nable::endpoints::HostInfo,
                $($param: &$param_type),*
            ) -> anyhow::Result<crate::endpoints::nable::structs::xml::XMLResult<$ret>> {
                let url = format!(
                    "https://{endpoint}/api/?apikey={api_key}&service={service}{params}",
                    endpoint = host_info.endpoint,
                    api_key = host_info.api_key.0,
                    service = stringify!($service),
                    params = {
                        let mut params = String::new();
                        $(params.push_str(&format!("{}={}&", stringify!($param), $param.to_string()));)*

                        if params.ends_with('&') {
                            params.pop();
                        }

                        if params.is_empty() {
                            params
                        } else {
                            format!("&{}", params)
                        }
                    }
                );
                tracing::debug!("url: {url}");

                let url = reqwest::Url::parse(&url)?;
                let request = client.get(url);

                let response = request.send().await?;
                let status = response.status();

                if !status.is_success() {
                    let text = response.text().await?;
                    anyhow::bail!("Failed to get response from server: {}", text);
                }

                let content = response.text().await?;

                tracing::debug!("content: {content}");

                quick_xml::de::from_str::<crate::endpoints::nable::structs::xml::XMLResult<$ret>>(&content).map_err(|err| {
                    tracing::error!("Failed to parse response from server: {err}");
                    err.into()
                })
            }
        };
    }
}
