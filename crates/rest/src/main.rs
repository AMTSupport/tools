#![feature(async_closure)]
#![feature(lazy_cell)]

use anyhow::{Context, Result};
use async_lazy::Lazy;
use chrono::Duration;
use clap::{Parser, Subcommand};
use lib::cli::Flags;
use lib::log::init;
use rest::hudu::web::Hudu;
use rest::hudu::HuduCommands;
use rest::nable::web::NAble;
use rest::{Client, Url};
use simplelog::{info, trace};

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    subcommand: Commands,
}

// TODO :: Verify endpoints and api keys are valid.
#[derive(Subcommand)]
enum Commands {
    Hudu {
        #[arg(short, long)]
        endpoint: String,
        #[arg(short, long)]
        api_key: String,
        #[command(flatten)]
        flags: Flags,
        #[command(subcommand)]
        subcommand: HuduCommands,
    },
    Nable {
        #[arg(short, long)]
        endpoint: String,
        #[arg(short, long)]
        api_key: String,
        #[command(flatten)]
        flags: Flags,
    },
}

pub enum ManagementType {
    Billable,
    Managed,
    Services,
    Unknown,
}

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
    let cli = Cli::try_parse().context("Parse CLI")?;
    let rules = Rules::default(); // TODO load from config

    match cli.subcommand {
        Commands::Hudu {
            endpoint,
            api_key,
            flags,
            subcommand,
        } => {
            let hudu = Client::hudu(&endpoint, &api_key)?;
            match &subcommand {
                HuduCommands::Query { outdated_passwords } => {
                    init("hudu-query", &flags).context("Init logging")?;
                    info!("Querying Hudu.");

                    let companies = Lazy::new(|| Box::pin(async { hudu.get_companies().await }));
                    let mut printed = false;
                    let maybe_newline = |printed: &mut bool| {
                        if *printed {
                            println!();
                        }
                    };

                    if *outdated_passwords {
                        maybe_newline(&mut printed);
                        info!("Querying for outdated passwords.");
                        let instant = chrono::Utc::now();
                        let companies = companies.force().await.as_ref().unwrap();

                        let passwords = hudu.get_passwords(&companies).await.unwrap();
                        let outdated_passwords = passwords
                            .iter()
                            .filter(|password| password.identity_company_id.is_some_and(|id| companies.contains_key(&id)))
                            .filter(|password| password.identity_name.contains("localadmin"))
                            .filter(|password| {
                                instant - password.meta_updated_at >= rules.password_max_age
                            })
                            .map(|password| {
                                trace!("Formatting password: {password:#?}");
                                format!(
                                    "\tClient <yellow>{client}<//>, updated <red>{days_since}<//> days ago. <green>{link}<//>",
                                    client = companies[&password.identity_company_id.unwrap()].name,
                                    days_since = (instant - password.meta_updated_at).num_days(),
                                    link = password.link(&hudu)
                                )
                            })
                            .collect::<Vec<String>>();

                        let log = format!(
                            "There are {count} outdated passwords for localadmin accounts;\nPasswords older than {days} days have been considered outdated.\n{passwords:#}",
                            count = outdated_passwords.len(),
                            days = rules.password_max_age.num_days(),
                            passwords = outdated_passwords.join("\n")
                        );

                        info!("{log}");
                        printed = true;
                    }
                }
            }
        }
        Commands::Nable {
            endpoint,
            api_key,
            flags,
        } => {
            init("nable", &flags).context("Init logging")?;
            let nable = Client::nable(&endpoint, &api_key)?;
            let clients = nable.get_clients().await?;

            info!("{clients:#?}")
        }
    }

    Ok(())
}
