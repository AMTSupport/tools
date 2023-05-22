use crate::nable::{API_ENDPOINT, SERVICE_LIST_CLIENTS};
use crate::Client;
use anyhow::{Context, Result};
use async_trait::async_trait;
use reqwest::RequestBuilder;
use simplelog::info;

type Clients = Vec<Client>;
// type Devices = Vec<Device>;

#[async_trait]
pub trait NAble: Sized {
    fn nable(base_url: &str, api_key: &str) -> Result<Self>;

    fn prepare_request(&self, service: &str) -> RequestBuilder;

    async fn get_clients(&self) -> Result<Clients> {
        let request = self.prepare_request(SERVICE_LIST_CLIENTS);
        let response = request
            .send()
            .await
            .context("Send rest request for clients")?;

        info!("Response: {:?}", response.text().await?);

        // Ok(request.send().await?.json::<Clients>().await?)
        Ok(vec![])
    }
}

impl NAble for Client {
    fn nable(base_url: &str, api_key: &str) -> Result<Self> {
        Ok(Self {
            base_url: format!("{base_url}{endpoint}", endpoint = API_ENDPOINT),
            api_key: api_key.to_string(),
            client: reqwest::Client::builder()
                .user_agent("rest")
                .gzip(true)
                .build()?,
        })
    }

    fn prepare_request(&self, service: &str) -> RequestBuilder {
        self.client.get(format!(
            "{url}?apikey={key}&service={service}",
            url = &self.base_url,
            key = &self.api_key,
        ))
    }
}
