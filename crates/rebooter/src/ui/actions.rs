use crate::reason;
use clap::Subcommand;
use reason::Reason;

#[derive(Subcommand)]
pub enum Action {
    /// Test if the reason would be valid and require a reboot.
    Test(Reason),
}
