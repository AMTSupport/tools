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

use anyhow::Result;
use clap::{Parser, Subcommand, ValueEnum};
use csv::{Writer, WriterBuilder};
use endpoints::nable::endpoints::NSightEndpoint;
use lib::cli::Flags;
use lib::log;
use macros::{EnumNames, EnumVariants};
use rest::endpoints;
use rest::endpoints::endpoint::Endpoint;
use rest::endpoints::nable::endpoints::Response;
use rest::endpoints::nable::structs::xml::Items;
use serde::{Serialize, Serializer};
use std::fmt::Debug;
use std::fs::File;
use tracing::{debug, error, info};

#[derive(Debug, Parser)]
struct Cli {
    #[command(subcommand)]
    endpoint: Endpoints,

    #[command(flatten)]
    flags: Flags,
}

#[derive(Debug, Subcommand)]
enum Endpoints {
    Nable {
        #[command(flatten)]
        args: <NSightEndpoint as Endpoint>::Args,

        #[command(subcommand)]
        request: <NSightEndpoint as Endpoint>::Request,

        #[arg(short, long, default_value = "stdout")]
        output: Output,
    },
    Interactive,
}

#[derive(Default, Debug, Clone, ValueEnum, EnumVariants, EnumNames)]
enum Output {
    #[default]
    Stdout,
    // Json,
    // PrettyJson,
    CSV,
}

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let _ = log::init("", cli.flags);

    match cli.endpoint {
        Endpoints::Nable { args, request, output } => {
            // use endpoints::nable::driver::Driver;

            // let driver = Driver::new().await?;
            // driver.deploy_templates("8319").await?;

            let instance = NSightEndpoint::new(args);
            let response = instance.handle(request).await?;

            match output {
                Output::Stdout => {
                    info!("{:#?}", response);
                }
                Output::CSV => {
                    let file = File::create("output.csv").unwrap();
                    let writer = WriterBuilder::new().has_headers(true).from_writer(file);
                    fn write(mut writer: Writer<File>, items: &[impl Serialize + Debug]) {
                        if items.is_empty() {
                            return;
                        }

                        items.iter().for_each(|i| {
                            debug!("Serializing: {:#?}", i);
                            writer.serialize(i).unwrap();
                        });

                        writer.flush().unwrap();
                    };

                    match response {
                        Response::Clients(clients) => write(writer, &clients.items.items),
                        Response::Sites(sites) => write(writer, &sites.items.items),
                        Response::Templates(templates) => write(writer, &templates.items.items),
                        Response::Servers(servers) => write(writer, &servers.items.items),
                        Response::Workstations(workstations) => write(writer, &workstations.items.items),
                    };
                }
            }
        }
        _ => todo!("Implement interactive mode"),
        // Endpoints::Interactive => {
        //     struct InteractiveConfiguration {
        //         running: bool,
        //         output: Output,
        //         history: Vec<String>,
        //     }
        //
        //     let mut config = InteractiveConfiguration {
        //         running: true,
        //         output: Output::default(),
        //         history: Vec::new(),
        //     };
        //
        //     while config.running {
        //         let input = inquire::Text::new("Command: ").prompt()?;
        //         match input.to_lowercase().as_str() {
        //             "exit" | "q" | "quit" | "close" => {
        //                 running = false;
        //             }
        //             "help" | "h" | "?" => {
        //                 info!(r#"Commands:
        //                 "#\texit, q, quit, close - Exits the program.");
        //                 println!("\thelp, h, ? - Displays this help message.");
        //                 println!("\tquery, q - Queries the Nable API.");
        //             }
        //             "history" | "hist" => info!("History: {}", config.history.join("\n")),
        //             "output" => {
        //                 let output = inquire::Select::new("Output: ", Output::get_variants()).prompt()?;
        //                 config.output = output;
        //                 info!("Output set to {output:#?}", output = config.output);
        //             }
        //             _ => {
        //                 error!("Unknown command.");
        //                 continue;
        //             }
        //         }
        //
        //         config.history.push(input);
        //     }
        // }
    }

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
