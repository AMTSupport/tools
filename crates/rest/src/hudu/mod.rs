pub mod runners;
pub mod structs;
pub mod web;

pub const API_ENDPOINT: &str = "/api/v1";
pub const API_HEADER: &str = "X-Api-Key";
pub const COMPANIES_ENDPOINT: &str = "/companies";
pub const PASSWORDS_ENDPOINT: &str = "/asset_passwords";

#[derive(clap::Subcommand)]
pub enum HuduCommands {
    Query {
        #[arg(short, long)]
        outdated_passwords: bool,
    },
}
