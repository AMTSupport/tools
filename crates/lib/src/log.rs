use crate::cli::Flags;
use anyhow::Context;
use simplelog::{
    trace, ColorChoice, CombinedLogger, ConfigBuilder, Level, LevelFilter, TermLogger,
    TerminalMode, ThreadLogMode, WriteLogger,
};
use std::env::temp_dir;
use std::fs::File;
use std::os::unix::fs::OpenOptionsExt;

pub fn init(named: &str, cli: &Flags) -> anyhow::Result<()> {
    let log_level = match cli.verbose {
        0 => LevelFilter::Info,
        1 => LevelFilter::Debug,
        _ => LevelFilter::Trace,
    };

    let log_file = temp_dir().join(format!("{0}.log", named));
    println!("Log file: {0}", log_file.display());

    CombinedLogger::init(vec![
        TermLogger::new(
            log_level,
            ConfigBuilder::new()
                .set_max_level(log_level)
                .set_time_level(LevelFilter::Off)
                .set_level_color(Level::Error, Some(simplelog::Color::Red))
                .set_level_color(Level::Warn, Some(simplelog::Color::Yellow))
                .set_level_color(Level::Trace, Some(simplelog::Color::Cyan))
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
            File::options()
                .append(true)
                .create(true)
                .mode(0o666)
                .open(log_file)
                .context("Create log file")?,
        ),
    ])
    .expect("Initialise Global Loggers");

    #[cfg(unix)]
    systemd_journal_logger::JournalLog::empty()
        .with_extra_fields(vec![("VERSION", env!("CARGO_PKG_VERSION"))])
        .with_syslog_identifier(named.to_string());

    Ok(())
}
