use crate::cli;
use anyhow::Context;
use log::LevelFilter;
use simplelog::{
    ColorChoice, CombinedLogger, ConfigBuilder, TermLogger, TerminalMode, ThreadLogMode,
    WriteLogger,
};
use std::env::temp_dir;
use std::fs::File;

pub fn init(named: &str, cli: &cli::Flags) -> anyhow::Result<()> {
    let log_level = match cli.verbose {
        0 => LevelFilter::Info,
        1 => LevelFilter::Debug,
        _ => LevelFilter::Trace,
    };

    CombinedLogger::init(vec![
        TermLogger::new(
            log_level,
            ConfigBuilder::new()
                .set_max_level(log_level)
                .set_thread_mode(ThreadLogMode::Names)
                .set_time_level(LevelFilter::Off)
                .build(),
            TerminalMode::Mixed,
            ColorChoice::Auto,
        ),
        WriteLogger::new(
            LevelFilter::Trace,
            ConfigBuilder::new()
                .set_max_level(LevelFilter::Trace)
                .set_thread_mode(ThreadLogMode::Names)
                .set_time_format_rfc2822()
                .build(),
            File::create(temp_dir().join(format!("{0}.log", named))).context("Create log file")?,
        ),
    ])
    .expect("Initialise Global Loggers");

    #[cfg(unix)]
    systemd_journal_logger::JournalLog::empty()
        .with_extra_fields(vec![("VERSION", env!("CARGO_PKG_VERSION"))])
        .with_syslog_identifier(named.to_string());

    Ok(())

    // .install()
    // .context("Install Journal Logger")
}
