use crate::cli::Flags;
use anyhow::Context;
use simplelog::ColorChoice;
use simplelog::CombinedLogger;
use simplelog::ConfigBuilder;
use simplelog::Level;
use simplelog::LevelFilter;
use simplelog::TermLogger;
use simplelog::TerminalMode;
use simplelog::ThreadLogMode;
use simplelog::WriteLogger;
use std::env::temp_dir;
use std::fs::File;

#[cfg(unix)]
use std::os::unix::fs::OpenOptionsExt;

pub fn init(named: &str, cli: &Flags) -> anyhow::Result<()> {
    let log_level = match cli.verbose {
        0 => LevelFilter::Info,
        1 => LevelFilter::Debug,
        _ => LevelFilter::Trace,
    };

    let log_file = temp_dir().join(format!("{0}.log", named));
    println!("Log file: {0}", log_file.display());

    let mut file_options = File::options();
    file_options.append(true).create(true);
    #[cfg(unix)]
    file_options.mode(0o666);

    let log_file = file_options.open(log_file).context("Create/Open log file")?;

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
            log_file,
        ),
    ])
    .expect("Initialise Global Loggers");

    #[cfg(unix)]
    systemd_journal_logger::JournalLog::empty()
        .with_extra_fields(vec![("VERSION", env!("CARGO_PKG_VERSION"))])
        .with_syslog_identifier(named.to_string());

    tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(tracing::Level::TRACE)
        .pretty()
        .with_thread_ids(true)
        .with_line_number(true)
        .with_file(true)
        .with_level(true)
        .finish();

    Ok(())
}
