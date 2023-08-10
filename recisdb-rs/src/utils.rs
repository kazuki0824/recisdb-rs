use chrono::Local;
use colored::*;
use env_logger::{Builder, Env};
use log::{Level, info};
use std::io::Write;

pub(crate) enum StreamExitType {
    Success(u64),
    Timeout,
    Error(std::io::Error),
    UnexpectedEofInTuner,
}

pub(crate) fn initialize_logger() {
    Builder::from_env(Env::default().default_filter_or("info"))
        .format(|buf, record| {
            let local_time = Local::now().format("%Y/%m/%d %H:%M:%S");
            let level = match record.level() {
                Level::Error => "ERROR".red(),
                Level::Warn  => "WARNING".yellow(),
                Level::Info  => "INFO".green(),
                Level::Debug => "DEBUG".cyan(),
                Level::Trace => "TRACE".blue(),
            };
            let level_padding = match record.level() {
                Level::Error => ":  ",
                Level::Warn  => ":",
                Level::Info  => ":   ",
                Level::Debug => ":  ",
                Level::Trace => ":  ",
            };
            writeln!(buf, "[{}] {}{}  {}", local_time, level, level_padding, record.args())
        })
        .init();
    info!("recisdb version {}", env!("CARGO_PKG_VERSION"));
}
