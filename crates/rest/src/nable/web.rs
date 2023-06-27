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

use crate::nable::structs::xml::XMLResult;
use crate::{
    nable::{structs::client::Clients, API_ENDPOINT, SERVICE_LIST_CLIENTS},
    Client as RestClient,
};
use anyhow::{Context, Result};
use async_trait::async_trait;
use http_cache_reqwest::{Cache, CacheMode, HttpCache};
use quick_xml::de::from_str;
use reqwest_middleware::RequestBuilder;

#[async_trait]
pub trait NAble: Sized {
    fn nable(base_url: &str, api_key: &str) -> Result<Self>;

    fn prepare_request(&self, service: &str) -> RequestBuilder;

    async fn get_clients(&self) -> Result<Clients>;
}

#[async_trait]
impl NAble for RestClient {
    fn nable(base_url: &str, api_key: &str) -> Result<Self> {
        let base_client = reqwest::Client::builder().user_agent(crate::AGENT).gzip(true).build()?;

        let client = reqwest_middleware::ClientBuilder::new(base_client)
            .with(Cache(HttpCache {
                mode: CacheMode::ForceCache,
                manager: http_cache_reqwest::MokaManager::default(),
                options: None,
            }))
            .build();

        Ok(Self {
            base_url: format!("{base_url}{endpoint}", endpoint = API_ENDPOINT),
            api_key: api_key.to_string(),
            client,
        })
    }

    fn prepare_request(&self, service: &str) -> RequestBuilder {
        self.client.get(format!(
            "{url}?apikey={key}&service={service}",
            url = &self.base_url,
            key = &self.api_key,
        ))
    }

    async fn get_clients(&self) -> Result<Clients> {
        let request = NAble::prepare_request(self, SERVICE_LIST_CLIENTS);
        let response = request
            .send()
            .await
            .context("Send rest request for clients")?
            .text()
            .await
            .context("Get raw text")?;

        let result: XMLResult<Clients> =
            from_str(&response).context("Deserialise clients from xml")?;
        Ok(result.items.clients)
    }
}
