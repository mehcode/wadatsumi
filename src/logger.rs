use chrono;
use env_logger::LogBuilder;
use log::{LogLevel, LogLevelFilter, SetLoggerError};
use ansi_term::Colour;

pub fn init(level: LogLevelFilter) -> Result<(), SetLoggerError> {
    let mut log_builder = LogBuilder::new();
    log_builder.format(|record| {
        const LOG_LEVEL_SHORT_NAMES: [&'static str; 6] =
            ["OFF", "ERRO", "WARN", "INFO", "DEBG", "TRCE"];

        let lvl = record.level();
        let lvl_color = match lvl {
            LogLevel::Error => 9,
            LogLevel::Warn => 3,
            LogLevel::Info => 2,
            LogLevel::Debug => 6,
            LogLevel::Trace => 4,
        };

        let lvl_s = LOG_LEVEL_SHORT_NAMES[lvl as usize];

        format!(
            "{} {} {}",
            chrono::Local::now().format("%H:%M:%S"),
            // record.target(),
            Colour::Fixed(lvl_color).paint(lvl_s),
            Colour::Fixed(15).paint(record.args().to_string()),
        )
    });

    // TODO: Allow configuring from a cli option
    log_builder.filter(Some("wadatsumi"), level);

    log_builder.init()
}
