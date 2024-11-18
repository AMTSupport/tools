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
use crate::endpoints::nable::endpoints::{HostInfo, NSightEndpoint};
use crate::endpoints::nable::structs::site::Site;
use crate::endpoints::nable::structs::template::Template;
use crate::endpoints::nable::NSightApiKey;
use anyhow::Result;
use reqwest::{Client, ClientBuilder, RequestBuilder};
use std::cell::LazyCell;
use std::collections::HashMap;
use thirtyfour::prelude::*;
use thiserror::Error;
use tracing::info;

pub struct Driver {
    reqwest: Client,

    driver: WebDriver,

    endpoint: NSightEndpoint,

    // TODO :: Securely store these.
    password: String,
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("Unable to login using provided credentials, please try again.")]
    UnableToLogin,

    #[error("Unknown error: {0}")]
    Unknown(#[from] anyhow::Error),
}

impl Driver {
    const fn dashboard_origin(&self) -> &'static str {
        *LazyCell::new(|| &*format!("https://{}", self.endpoint.host.endpoint))
    }
    const fn dashboard_refererr(&self) -> &'static str {
        *LazyCell::new(|| &*format!("{}/default.php", self.dashboard_origin()))
    }

    pub async fn new<S: AsRef<str>>(endpoint: S) -> Result<Self> {
        use inquire::{Password, Text};

        // TODO :: OS and installed browser detection.
        let driver_capabilities = DesiredCapabilities::firefox();
        let driver = WebDriver::new("http://localhost:4444", driver_capabilities).await?;

        driver.goto(&*format!("https://{endpoint}")).await?;

        async fn input_next(
            driver: &WebDriver,
            name: &str,
            input: impl AsRef<str> + Sized,
            by: impl Into<By> + Sized,
        ) -> Result<()> {
            let field = driver.find(By::Id(&*format!("{name}-field"))).await?;
            field.send_keys(input).await?;
            driver.find(by).await?.click().await?;
            Ok(())
        }

        let input_email = Text::new("Please enter your email: ").prompt()?;
        input_next(&driver, "email", &input_email, By::ClassName("login-next-button")).await?;

        let input_password = Password::new("Please enter your password: ").without_confirmation().prompt()?;
        input_next(&driver, "password", &input_password, By::ClassName("login-next-button")).await?;

        let input_2fa = Text::new("Please enter your 2FA code: ").prompt()?;
        input_next(&driver, "code", &input_2fa, By::Id("verify-submit")).await?;

        let reqwest = ClientBuilder::new()
            .user_agent(
                driver
                    .execute("return navigator.userAgent", Vec::new())
                    .await?
                    .json()
                    .as_str()
                    .unwrap(),
            )
            .gzip(true)
            .build()?;

        // TODO :: API KEY
        let endpoint = NSightEndpoint::new(HostInfo {
            endpoint,
            api_key: NSightApiKey::new("")?,
        })?;

        Ok(Self {
            reqwest,
            driver,
            endpoint,
            password: input_password,
        })
    }

    pub async fn deploy_all_templates(self) -> Result<()> {
        // let clients = NSightEndpoint

        Ok(())
    }

    pub async fn deploy_template(self, client_id: &Client, site_id: Option<&Site>, template: &Template) -> Result<()> {
        let request = self.post_request().await?.form(&HashMap::from([
            (
                "function",
                "create_apply_monitoring_template_device_action_for_client_or_site",
            ),
            ("action", "createAction"),
            ("dashaction", "24"),
            ("data", "33455"),
            ("password", &self.password),
            ("confirmed", "false"),
            ("applyMonitoringTemplateFromEntityTree", "true"),
            ("siteid", "193840"),
            ("clientid", "8240"),
            ("isWorkstation", "true"),
        ]));

        info!("request: {:?}", request);
        let request = request.send().await?;
        info!("Response: {:?}", request.text().await?);
        let input = std::io::stdin();
        let mut line = String::new();
        input.read_line(&mut line).unwrap();

        self.driver.quit().await?;

        Ok(())
    }

    async fn post_request(&self) -> Result<RequestBuilder> {
        let cookies = self
            .driver
            .get_all_cookies()
            .await?
            .into_iter()
            .map(Cookie::to_string)
            .collect::<Vec<String>>()
            .join("; ");

        Ok(self
            .reqwest
            .post("https://dashboard.amt.com.au/data_processor.php")
            .header("Host", &*self.dashboard_endpoint)
            .header("Origin", &self.dashboard_origin())
            .header("Referer", &self.dashboard_refererr())
            .header("Cookie", cookies))
    }
}
