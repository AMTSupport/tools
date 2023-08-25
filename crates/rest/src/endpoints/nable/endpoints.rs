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
use crate::endpoints::nable::structs::site::Sites;
use crate::endpoints::nable::structs::template::Templates;
use crate::endpoints::nable::structs::xml::XMLResult;
use crate::rest;
use anyhow::Context;
use clap::{Parser, Subcommand};
use reqwest::Client;
use reqwest_middleware::ClientWithMiddleware;
use tracing::instrument;
use crate::endpoints::nable::structs::device::{Servers, Workstations};

// Endpoints not implemented:
// Get Client Sites (by client id or client name or just all sites)
// Get default Site Monitoring template
// Apply Site Monitoring template to Site

#[derive(Debug)]
pub struct NSightEndpoint {
    host: HostInfo,
    client: ClientWithMiddleware,
}

#[derive(Debug, Subcommand)]
pub enum Request {
    GetClients,
    GetSites { clientid: String },
    GetTemplates,

    GetServers { siteid: String },
    GetWorkstations { siteid: String },
}

#[derive(Debug)]
pub enum Response {
    Clients(XMLResult<Clients>),
    Sites(XMLResult<Sites>),
    Templates(XMLResult<Templates>),
    Servers(XMLResult<Servers>),
    Workstations(XMLResult<Workstations>),
}

#[derive(Debug, Parser)]
pub struct HostInfo {
    pub endpoint: String,
    pub api_key: String,
}

impl Endpoint for NSightEndpoint {
    type Args = HostInfo;
    type Request = Request;
    type Response = Response;

    // TODO :: validate api key
    #[instrument]
    fn new(args: Self::Args) -> Self {
        use http_cache_reqwest::{Cache, CacheMode, HttpCache, MokaManager};
        use reqwest::Client;
        use reqwest_middleware::ClientBuilder;

        let builder = Client::builder().gzip(true);
        let client = ClientBuilder::new(builder.build().unwrap())
            .with(Cache(HttpCache {
                mode: CacheMode::ForceCache,
                manager: MokaManager::default(),
                options: None,
            }))
            .build();

        Self { host: args, client }
    }

    #[instrument]
    async fn handle(&self, event: Self::Request) -> anyhow::Result<Self::Response> {
        use Request::*;

        match event {
            GetClients => {
                list_clients(&self.client, &self.host.endpoint, &self.host.api_key).await.map(Response::Clients)
            }
            GetSites { clientid } => {
                list_sites(&self.client, &self.host.endpoint, &self.host.api_key, &clientid).await.map(Response::Sites)
            }
            GetTemplates => {
                list_templates(&self.client, &self.host.endpoint, &self.host.api_key).await.map(Response::Templates)
            }
            GetServers { siteid } => {
                list_servers(&self.client, &self.host.endpoint, &self.host.api_key, &siteid).await.map(Response::Servers)
            }
            GetWorkstations { siteid } => {
                list_workstations(&self.client, &self.host.endpoint, &self.host.api_key, &siteid).await.map(Response::Workstations)
            }
        }
    }
}

rest!(list_clients => Clients);
rest!(list_sites & [clientid] => Sites);
rest!(list_templates => Templates);

rest!(list_servers & [siteid] => Servers);
rest!(list_workstations & [siteid] => Workstations);

// rest!(list_servers & [siteid] => Clients);
// rest!(list_worksations & [siteid] => Clients);

mod macros {
    #[macro_export]
    macro_rules! rest {
        ($service:ident => $ret:ty) => (rest!($service & [] => $ret););
        ($service:ident & [$($params:ident),*] => $ret:ty) => {
            #[tracing::instrument(skip(client, api_key))]
            pub async fn $service(
                client: &reqwest_middleware::ClientWithMiddleware,
                endpoint: &str,
                api_key: &str,
                $($params: &str),*
            ) -> anyhow::Result<crate::endpoints::nable::structs::xml::XMLResult<$ret>> {
                let url = format!(
                    "https://{endpoint}/api/?apikey={api_key}&service={service}{params}",
                    service = stringify!($service),
                    params = {
                        let mut params = String::new();
                        $(params.push_str(&format!("{}={}&", stringify!($params), $params));)*

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

                let url = reqwest::Url::parse(&url)?;
                let request = client.get(url);

                let response = request.send().await?;
                let status = response.status();

                if !status.is_success() {
                    let text = response.text().await?;
                    tracing::error!("Failed to get response from server: {}", text);
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
