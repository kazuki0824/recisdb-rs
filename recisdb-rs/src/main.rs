use crate::utils::StreamExitType;
use clap::Parser;
use futures_executor::block_on;
use futures_time::future::FutureExt;
use log::info;

mod channels;
mod commands;
mod context;
mod io;
mod tuner;
mod utils;


fn main() {
    let arg = context::Cli::parse();
    info!("{:?}", arg);

    utils::initialize_logger();

    let result = match commands::process_command(arg) {
        (fut, None) => {
            block_on(fut).map_or_else(StreamExitType::Error, StreamExitType::Success)
        }
        (fut, Some(dur)) => match block_on(fut.timeout(dur)) {
            Ok(Ok(_)) => StreamExitType::UnexpectedEofInTuner,
            Ok(Err(e)) => StreamExitType::Error(e),
            _ => StreamExitType::Timeout,
        },
    };

    match result {
        StreamExitType::Success(_) => {}
        StreamExitType::Timeout => {}
        StreamExitType::Error(_) => {}
        StreamExitType::UnexpectedEofInTuner => {}
    }
}

