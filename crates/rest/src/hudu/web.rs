use crate::hudu::structs::company::{Companies, Company};
use crate::hudu::structs::password::{Password, Passwords};
use crate::hudu::{API_ENDPOINT, API_HEADER, COMPANIES_ENDPOINT, PASSWORDS_ENDPOINT};
use anyhow::{Context, Result};
use async_trait::async_trait;
use reqwest::{
    header::{self, HeaderMap},
    RequestBuilder,
};
use serde::de::DeserializeOwned;
use simplelog::trace;
use std::collections::HashMap;

#[async_trait]
pub trait Hudu: Sized {
    fn prepare_request(&self, uri: &str) -> RequestBuilder;

    async fn paginated_request<I>(&self, builder: RequestBuilder) -> Result<Vec<I>>
    where
        I: DeserializeOwned + Send,
    {
        let builder = builder.query(&[("page_size", 1000)]);
        let start = std::time::Instant::now();

        let mut page = 1;
        let mut results = Vec::new();

        // TODO :: Stream
        loop {
            let builder = builder
                .try_clone()
                .context("Clone request builder")?
                .query(&[("page", &page)]);

            let response = builder
                .send()
                .await
                .context(format!("Send paginated request for page {page}"))?;

            #[derive(serde::Deserialize)]
            struct VecHandler<I> {
                #[serde(alias = "companies", alias = "asset_passwords")]
                // TODO :: is there any way to fix having to hardcode these names?
                vec: Vec<I>,
            }

            let handler = response
                .json::<VecHandler<I>>()
                .await
                .context("Deserialise response")?;

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

    fn hudu(base_url: &str, api_key: &str) -> Result<Self>;

    async fn get_companies(&self) -> Result<Companies>;
    async fn get_passwords(&self, companies: &Companies) -> Result<Passwords>;
    async fn get_company(&self, id: &u8) -> Result<Company>;
}

#[async_trait]
impl Hudu for crate::Client {
    fn prepare_request(&self, uri: &str) -> RequestBuilder {
        self.client.get(format!("{0}{uri}", &self.base_url))
    }

    fn hudu(base_url: &str, api_key: &str) -> Result<Self> {
        let mut headers = HeaderMap::with_capacity(2);
        headers.insert(API_HEADER, api_key.parse()?);
        headers.insert(header::ACCEPT, "application/json; charset=utf-8".parse()?);

        Ok(Self {
            base_url: format!("{base_url}{endpoint}", endpoint = API_ENDPOINT),
            api_key: api_key.to_string(),
            client: reqwest::Client::builder()
                .user_agent("rest")
                .default_headers(headers)
                .gzip(true)
                .build()?,
        })
    }

    async fn get_companies(&self) -> Result<Companies> {
        let request = self.prepare_request(COMPANIES_ENDPOINT);
        let companies = self.paginated_request::<Company>(request).await?;
        let map = companies
            .into_iter()
            .map(|company| (company.id.clone(), company))
            .collect::<HashMap<usize, Company>>();

        trace!("Got {:#?} companies", map.len());
        trace!("Companies: {map:#?}");

        Ok(map)
    }

    async fn get_passwords(&self, _companies: &Companies) -> Result<Passwords> {
        let request = self.prepare_request(PASSWORDS_ENDPOINT);
        let passwords = self.paginated_request::<Password>(request).await?;

        // TODO :: Filter companies

        Ok(passwords)
    }

    async fn get_company(&self, id: &u8) -> Result<Company> {
        let response: Company = self
            .prepare_request(&format!("{COMPANIES_ENDPOINT}/{id}"))
            .send()
            .await
            .context(format!("Send rest request for company {id}"))?
            .json()
            .await
            .context("Parse json for company")?;

        Ok(response)
    }
}
