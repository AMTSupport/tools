use anyhow::Context;
use log::LevelFilter;
use simplelog::{
    ColorChoice, CombinedLogger, Config, ConfigBuilder, TermLogger, TerminalMode, ThreadLogMode,
    WriteLogger,
};
use std::env::temp_dir;
use std::fs::File;
use std::string::ToString;
use systemd_journal_logger::JournalLog;

const LOGGER_FILE: &str = &"{0}.log";

fn create_config() -> Config {
    return ConfigBuilder::new()
        .set_max_level(LevelFilter::Debug)
        .set_thread_mode(ThreadLogMode::Names)
        .set_time_format_rfc2822()
        .build();
}

pub fn init_loggers(named: &str) -> anyhow::Result<()> {
    let config = create_config();

    #[cfg(not(target_os = "windows"))]
    JournalLog::default()
        .with_extra_fields(vec![("VERSION", env!("CARGO_PKG_VERSION"))])
        .with_syslog_identifier(named.to_string())
        .install()
        .unwrap();

    CombinedLogger::init(vec![
        TermLogger::new(
            LevelFilter::Info,
            config.to_owned(),
            TerminalMode::Mixed,
            ColorChoice::Auto,
        ),
        WriteLogger::new(
            LevelFilter::Trace,
            config,
            File::create(temp_dir().join(format!(LOGGER_FILE, named)))
                .context("Create log file")?,
        ),
    ])
    .context("Initialise Global Loggers")
}
