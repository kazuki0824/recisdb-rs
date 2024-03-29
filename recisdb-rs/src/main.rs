use crate::utils::StreamExitType;
use clap::Parser;
use futures_executor::block_on;
use futures_time::future::FutureExt;
use log::{debug, info};

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

    // Get Future
    let (fut, timeout_option, progress) = commands::process_command(arg);

    let result = {
        // Common code for handling progress
        if let Some((file_sz, rx)) = progress {
            std::thread::spawn(move || {
                let pb = utils::init_progress(file_sz);

                loop {
                    match rx.recv() {
                        Ok(u64::MAX) => {
                            utils::progress(&pb, file_sz);
                            debug!("fill")
                        }
                        Ok(v) => {
                            utils::progress(&pb, v);
                        }
                        Err(e) => {
                            debug!("{}", e);
                            break;
                        }
                    }
                }
            });
        }

        // Handling the future based on the presence of a timeout
        match timeout_option {
            Some(dur) => match block_on(fut.timeout(dur)) {
                Ok(Ok(_)) => StreamExitType::UnexpectedEofInTuner,
                Ok(Err(e)) => StreamExitType::Error(e),
                _ => StreamExitType::Timeout,
            },
            None => block_on(fut).map_or_else(StreamExitType::Error, StreamExitType::Success),
        }
    };

    match result {
        StreamExitType::Success(_) => {}
        StreamExitType::Timeout => {}
        StreamExitType::Error(_) => {}
        StreamExitType::UnexpectedEofInTuner => {}
    }
}
