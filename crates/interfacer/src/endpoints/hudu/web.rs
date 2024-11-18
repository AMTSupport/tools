/*
 * Copyright (c) 2024. James Draycott <me@racci.dev>
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
 * You should have received a copy of the GNU General Public License along with this program.
 * If not, see <https://www.gnu.org/licenses/>.
 */

use crate::endpoints::endpoint::Endpoint;
use crate::endpoints::hudu::structs::company::{Companies, Company};
use crate::endpoints::hudu::structs::password::{Password, Passwords};
use crate::endpoints::hudu::{API_ENDPOINT, API_HEADER, COMPANIES_ENDPOINT, PASSWORDS_ENDPOINT};
use anyhow::{Context, Result};
use reqwest::{
    header::{self, HeaderMap},
    Response,
};
use serde::de::DeserializeOwned;
use std::any::Any;

use http_cache_reqwest::{Cache, CacheMode, HttpCache, HttpCacheOptions};
use reqwest_middleware::RequestBuilder;
use std::collections::HashMap;
use std::hash::Hash;
use tracing::{error, trace};

pub trait Hudu {
    fn hudu(base_url: &str, api_key: &str) -> Result<Self>
    where
        Self: Sized;

    fn prepare_request(&self, uri: &str) -> RequestBuilder
    where
        Self: Sized;

    async fn get_companies(&self) -> Result<Companies>
    where
        Self: Sized;
    async fn get_passwords(&self, companies: &Companies) -> Result<Passwords>
    where
        Self: Sized;

    async fn get_company(&self, id: &u8) -> Result<Company>
    where
        Self: Sized;
}

impl<T: Endpoint> Hudu for T
where
    Self: 'static + Send + Sync + Eq + Hash + Clone,
{
    fn hudu(base_url: &str, api_key: &str) -> Result<Self> {
        let mut headers = HeaderMap::with_capacity(2);
        headers.insert(API_HEADER, api_key.parse()?);
        headers.insert(header::ACCEPT, "application/json; charset=utf-8".parse()?);

        let base_client =
            reqwest::Client::builder().default_headers(headers).gzip(true).build()?;

        let client = reqwest_middleware::ClientBuilder::new(base_client)
            .with(Cache(HttpCache {
                mode: CacheMode::ForceCache,
                manager: http_cache_reqwest::MokaManager::default(),
                options: HttpCacheOptions::default(),
            }))
            .build();

        Ok(Self {
            base_url: format!("{base_url}{endpoint}", endpoint = API_ENDPOINT),
            api_key: api_key.to_string(),
            client,
        })
    }

    // TODO :: Cache
    fn prepare_request(&self, uri: &str) -> RequestBuilder {
        self.client.get(format!("{0}{uri}", &self.base_url))
    }

    async fn get_companies(&self) -> Result<Companies>
    where
        Companies: 'static + Clone + Send + Sync + Eq,
    {
        let request = self.prepare_request(COMPANIES_ENDPOINT);
        let companies = match paginated_request::<Company>(request).await {
            Ok(companies) => companies,
            Err(e) => {
                error!("Error getting companies: {:#?}", e);
                Vec::new()
            }
        };

        Ok(companies.into_iter().map(|company| (company.id, company)).collect::<Companies>())
    }

    async fn get_passwords(&self, _companies: &Companies) -> Result<Passwords> {
        let request = self.prepare_request(PASSWORDS_ENDPOINT);
        let passwords = paginated_request::<Password>(request).await?;

        // TODO :: Filter companies

        Ok(passwords)
    }

    async fn get_company(&self, id: &u8) -> Result<Company>
// where
        //     Self: 'static + Send + Sync + Eq,
        //     Option<Company>: 'static + Send + Sync
    {
        match self
            .prepare_request(&format!("{COMPANIES_ENDPOINT}/{id}"))
            .send()
            .await
            .context(format!("Send rest request for company {id}"))
            .and_then(check_auth)
        {
            Ok(response) => {
                response.json::<Company>().await.context(format!("Deserialise response for company {id}", id = id))
            }
            Err(e) => {
                error!("Error getting company {id}: {:#?}", e);
                Err(anyhow::anyhow!("Error getting company {id}", id = id))
            }
        }
    }
}

fn check_auth(response: Response) -> Result<Response> {
    if response.status().is_client_error() {
        return Err(anyhow::anyhow!("Unable to authenticate with Hudu"));
    }

    Ok(response)
}

thread_local! {
    static REQUESTS: HashMap<RequestBuilder, Vec<Box<dyn Any>>> = std::collections::HashMap::new();
}

async fn paginated_request<I>(builder: RequestBuilder) -> Result<Vec<I>>
where
    I: DeserializeOwned + Send,
{
    let builder = builder.query(&[("page_size", 1000)]);
    let start = std::time::Instant::now();

    let mut page = 1;
    let mut results = Vec::new();

    // TODO :: Stream
    loop {
        let builder = builder.try_clone().context("Clone request builder")?.query(&[("page", &page)]);

        let response = builder.send().await.context(format!("Send paginated request for page {page}"))?;

        #[derive(serde::Deserialize)]
        struct VecHandler<I> {
            #[serde(alias = "companies", alias = "asset_passwords")]
            // TODO :: is there any way to fix having to hardcode these names?
            vec: Vec<I>,
        }

        let handler = response.json::<VecHandler<I>>().await.context("Deserialise response")?;

        let mut vec = handler.vec;

        if vec.is_empty() {
            break;
        }

        results.append(&mut vec);
        page += 1;
    }
    trace!(
        "Took {:#?} to collect paginated results for {builder:#?}",
        start.elapsed()
    );

    Ok(results.into_iter().collect::<Vec<I>>())
}
