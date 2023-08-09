use crate::utils::StreamExitType;
use clap::Parser;
use futures_executor::block_on;
use futures_time::future::FutureExt;
use log::{error, info};

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
        (fut, None, progress) => {
            if let Some(rx) = progress {
                std::thread::spawn(move || loop {
                    match rx.recv() {
                        Ok(v) => {
                            utils::progress(v);
                        }
                        Err(e) => {
                            error!("{}", e);
                        }
                    }
                });
            }
            block_on(fut).map_or_else(StreamExitType::Error, StreamExitType::Success)
        }
        (fut, Some(dur), progress) => {
            if let Some(rx) = progress {
                std::thread::spawn(move || loop {
                    match rx.recv() {
                        Ok(v) => {
                            utils::progress(v);
                        }
                        Err(e) => {
                            error!("{}", e);
                        }
                    }
                });
            }
            match block_on(fut.timeout(dur)) {
                Ok(Ok(_)) => StreamExitType::UnexpectedEofInTuner,
                Ok(Err(e)) => StreamExitType::Error(e),
                _ => StreamExitType::Timeout,
            }
        }
    };

    match result {
        StreamExitType::Success(_) => {}
        StreamExitType::Timeout => {}
        StreamExitType::Error(_) => {}
        StreamExitType::UnexpectedEofInTuner => {}
    }
}
