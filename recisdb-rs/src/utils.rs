use chrono::Local;
use colored::*;
use env_logger::{Builder, Env};
use indicatif::{ProgressBar, ProgressStyle};
use log::{info, Level};
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
                Level::Warn => "WARNING".yellow(),
                Level::Info => "INFO".green(),
                Level::Debug => "DEBUG".cyan(),
                Level::Trace => "TRACE".blue(),
            };
            let level_padding = match record.level() {
                Level::Error => ":  ",
                Level::Warn => ":",
                Level::Info => ":   ",
                Level::Debug => ":  ",
                Level::Trace => ":  ",
            };
            writeln!(
                buf,
                "[{}] {}{}  {}",
                local_time,
                level,
                level_padding,
                record.args()
            )
        })
        .init();
    info!("recisdb version {}", env!("CARGO_PKG_VERSION"));
}

pub(crate) fn progress(bar: &ProgressBar, value: u64) {
    bar.set_position(value);
}

pub(crate) fn init_progress(max: u64) -> ProgressBar {
    // プログレスバーの長さを指定してプログレスバーを作成
    let pb = ProgressBar::new(max);
    // プログレスバーで表示する文字列を指定
    pb.set_style(
        ProgressStyle::default_bar()
            .template(
                "{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta})",
            )
            .unwrap(),
    );
    pb
}
