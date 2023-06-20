use tracing::error;
use sysexits::ExitCode;

const ERROR_MESSAGE: &str = "Failed to elevate privileges";

pub fn elevated_privileges() -> bool {
    #[cfg(windows)]
    return is_elevated::is_elevated();

    #[cfg(unix)]
    nix::unistd::geteuid().is_root()
}

pub fn required_elevated_privileges() -> Option<ExitCode> {
    let code = elevated_privileges();

    if !code {
        error!("{}", ERROR_MESSAGE);
        return Some(ExitCode::NoPerm);
    }

    None
}
