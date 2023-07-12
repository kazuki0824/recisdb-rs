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

#[cfg(target_os = "linux")]
fn handle_tuning_error(e: Box<dyn std::error::Error>) -> ! {
    if let Some(nix_err) = e.downcast_ref::<nix::Error>() {
        let current_errno = nix::errno::Errno::from_i32(nix::errno::errno());
        match current_errno {
            nix::errno::Errno::EAGAIN => {
                error!("Channel selection failed. The channel may not be received.");
            }
            nix::errno::Errno::EINVAL => {
                error!("The specified channel is invalid.");
            }
            _ => {
                error!("Unexpected Linux error: {}", nix_err);
            }
        }
    } else if let Some(io_error) = e.downcast_ref::<std::io::Error>() {
        if let Some(raw_os_error) = io_error.raw_os_error() {
            match raw_os_error {
                libc::EALREADY => {
                    error!("The tuner device is already in use.");
                }
                _ => {
                    error!("Unexpected IO error: {}", io_error);
                }
            }
        } else {
            error!("Unexpected IO error: {}", io_error);
        }
    } else {
        error!("Unexpected error: {}", e);
    }
    std::process::exit(1);
}

#[cfg(target_os = "windows")]
fn handle_tuning_error(e: Box<dyn std::error::Error>) -> ! {
    error!("Unexpected error: {}", e);
    std::process::exit(1);
}

fn main() {
    let arg = context::Cli::parse();
    info!("{:?}", arg);

    utils::initialize_logger();

    let result = match commands::process_command(arg) {
        (fut, None) => {
            block_on(fut).map_or_else(|e| StreamExitType::Error(e), |t| StreamExitType::Success(t))
        }
        (fut, Some(dur)) => match block_on(fut.timeout(dur)) {
            Ok(Ok(_)) => StreamExitType::UnexpectedEofInTuner,
            Ok(Err(e)) => StreamExitType::Error(e),
            _ => StreamExitType::Timeout,
        },
    };

    
}

