use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// The verbosity of the terminal logger
    #[arg(short, long, action = clap::ArgAction::Count)]
    pub verbose: u8,

    /// If there shouldn't be any changes made and only a dry run should be performed
    #[arg(short, long, action = clap::ArgAction::SetTrue)]
    pub dry_run: bool,
}

pub fn init() -> Cli {
    return Cli::parse();
}