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

use crate::nable::structs::client::Clients;
use crate::rest;
use anyhow::Context;

rest!(list_clients => Clients);
// rest!(list_sites & [clientid] => Sites);

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
            ) -> anyhow::Result<crate::nable::structs::xml::XMLResult<$ret>> {
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

                let url = reqwest::Url::parse(&url).context("Failed to parse URL")?;
                let request = client.get(url);

                use anyhow::Context;
                let response = request.send().await.context("Get response from server")?;
                let status = response.status();

                if !status.is_success() {
                    let text = response.text().await.context("Get response text")?;
                    tracing::error!("Failed to get response from server: {}", text);
                    anyhow::bail!("Failed to get response from server: {}", text);
                }

                let content = response.text().await.context("Get response text")?;

                tracing::debug!("content: {content}");

                quick_xml::de::from_str::<crate::nable::structs::xml::XMLResult<$ret>>(&content).context("Parse XML")
            }
        };
    }
}
