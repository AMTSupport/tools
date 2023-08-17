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

#![feature(async_closure)]
#![feature(lazy_cell)]
#![feature(async_fn_in_trait)]

use anyhow::Result;
use chrono::Duration;
use http_cache_reqwest::{Cache, CacheMode, HttpCache};
use reqwest::header::USER_AGENT;
use tracing::{debug, Level};

// #[derive(Parser)]
// struct Cli {
//     #[command(subcommand)]
//     subcommand: Commands,
// }
//
// // TODO :: Verify endpoints and api keys are valid.
// #[derive(Subcommand)]
// enum Commands {
//     Hudu {
//         #[arg(short, long)]
//         endpoint: String,
//         #[arg(short, long)]
//         api_key: String,
//         #[command(flatten)]
//         flags: Flags,
//         #[command(subcommand)]
//         subcommand: HuduCommands,
//     },
//     Nable {
//         #[arg(short, long)]
//         endpoint: String,
//         #[arg(short, long)]
//         api_key: String,
//         #[command(flatten)]
//         flags: Flags,
//     },
//     Manager {
//         #[arg(long)]
//         hudu_endpoint: String,
//         #[arg(long)]
//         hudu_api_key: String,
//         #[arg(long)]
//         nable_endpoint: String,
//         #[arg(long)]
//         nable_api_key: String,
//         #[command(flatten)]
//         flags: Flags,
//         #[command(subcommand)]
//         subcommand: ManagerCommands,
//     },
// }

pub struct Rules {
    password_max_age: Duration,
    // management_type: ManagementType,
}

impl Default for Rules {
    fn default() -> Self {
        Self {
            password_max_age: Duration::days(90),
            // management_type: ManagementType::Unknown,
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt().pretty().with_max_level(Level::DEBUG).init();
    // let cli = Cli::try_parse().context("Parse CLI")?;
    // let rules = Rules::default(); // TODO load from config

    let base_client = reqwest::Client::builder().user_agent(USER_AGENT).gzip(true).build()?;
    let client = reqwest_middleware::ClientBuilder::new(base_client)
        .with(Cache(HttpCache {
            mode: CacheMode::ForceCache,
            manager: http_cache_reqwest::MokaManager::default(),
            options: None,
        }))
        .build();

    let api_key = "ccad7b4188f49fddc3500eaec26f537a";
    let endpoint = "system-monitor.com";

    let clients = rest::nable::endpoints::list_clients(&client, &endpoint, &api_key).await?;
    println!("{:?}", clients);

    // match cli.subcommand {
    //     Commands::Hudu {
    //         endpoint,
    //         api_key,
    //         flags,
    //         subcommand,
    //     } => {
    //         let hudu = Client::hudu(&endpoint, &api_key)?;
    //         match &subcommand {
    //             HuduCommands::Query { outdated_passwords } => {
    //                 init("hudu-query", &flags).context("Init logging")?;
    //                 info!("Querying Hudu.");
    //
    //                 let companies = Lazy::new(|| Box::pin(async { hudu.get_companies().await }));
    //                 let mut printed = false;
    //                 let maybe_newline = |printed: &mut bool| {
    //                     if *printed {
    //                         println!();
    //                     }
    //                 };
    //
    //                 if *outdated_passwords {
    //                     maybe_newline(&mut printed);
    //                     info!("Querying for outdated passwords.");
    //                     let instant = chrono::Utc::now();
    //                     let companies = companies.force().await.as_ref().unwrap();
    //
    //                     let passwords = hudu.get_passwords(companies).await.unwrap();
    //                     let outdated_passwords = passwords
    //                         .iter()
    //                         .filter(|password| password.identity_company_id.is_some_and(|id| companies.contains_key(&id)))
    //                         .filter(|password| password.identity_name.contains("localadmin"))
    //                         .filter(|password| {
    //                             instant - password.meta_updated_at >= rules.password_max_age
    //                         })
    //                         .map(|password| {
    //                             trace!("Formatting password: {password:#?}");
    //                             format!(
    //                                 "\tClient <yellow>{client}<//>, updated <red>{days_since}<//> days ago. <green>{link}<//>",
    //                                 client = companies[&password.identity_company_id.unwrap()].name,
    //                                 days_since = (instant - password.meta_updated_at).num_days(),
    //                                 link = password.link(&hudu)
    //                             )
    //                         })
    //                         .collect::<Vec<String>>();
    //
    //                     let log = format!(
    //                         "There are {count} outdated passwords for localadmin accounts;\nPasswords older than {days} days have been considered outdated.\n{passwords:#}",
    //                         count = outdated_passwords.len(),
    //                         days = rules.password_max_age.num_days(),
    //                         passwords = outdated_passwords.join("\n")
    //                     );
    //
    //                     info!("{log}");
    //                     printed = true;
    //                 }
    //             }
    //         }
    //     }
    //     Commands::Nable {
    //         endpoint,
    //         api_key,
    //         flags,
    //     } => {
    //         init("nable", &flags).context("Init logging")?;
    //         let nable = Client::nable(&endpoint, &api_key)?;
    //         let clients = nable.get_clients().await?;
    //
    //         info!("{clients:#?}")
    //     }
    //     Commands::Manager {
    //         hudu_endpoint,
    //         hudu_api_key,
    //         nable_endpoint,
    //         nable_api_key,
    //         flags,
    //         subcommand,
    //     } => {
    //         init(format!("manager-{subcommand:#?}").as_str(), &flags).context("Init logging")?;
    //         let hudu = Client::hudu(&hudu_endpoint, &hudu_api_key)?;
    //         let nable = Client::nable(&nable_endpoint, &nable_api_key)?;
    //
    //         subcommand.run(hudu, nable).await.expect("Failed to run subcommand")
    //     }
    // }

    Ok(())
}
