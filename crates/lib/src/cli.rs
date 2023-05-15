use clap::Parser;

#[derive(Parse, Debug)]
#[command(auther, version, about, long_about = None)]
pub struct Flags {
    /// The verbosity of the terminal logger
    #[arg(short, long, parse(from_occurrences))]
    verbose: u8,

    /// If there shouldn't be any changes made and only a dry run should be performed
    #[arg(short, long, parse(from_flag))]
    dry_run: bool,
}
