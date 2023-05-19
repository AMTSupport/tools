use anyhow::Context;
use clap::{Parser, Subcommand};
use lib::cli::{Flags};
use lib::log;

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct ParentCli {
    // Which internal crate application to run.
    #[command(subcommand)]
    commands: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    #[command(arg_required_else_help = true)]
    Cleaner {
        #[command(flatten)]
        flags: Flags
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = ParentCli::try_parse().context("Parse CLI")?;

    match cli.commands {
        Commands::Cleaner { flags } => {
            log::init("cleaner", &flags)?;
            cleaner::application::application(flags).await?
        },
    }

    Ok(())
}
