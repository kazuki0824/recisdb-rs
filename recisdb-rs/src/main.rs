use crate::utils::StreamExitType;
use clap::Parser;
use futures_executor::block_on;
use futures_time::future::FutureExt;
use log::info;

#[cfg(any(target_os = "linux", target_os = "windows"))]
mod channels;
mod commands;
mod context;
mod io;
#[cfg(any(target_os = "linux", target_os = "windows"))]
mod tuner;
mod utils;

fn main() {
    let arg = context::Cli::parse();
    info!("{:?}", arg);

    utils::initialize_logger();

    // Get Future
    let (fut, timeout_option, progress) = commands::process_command(arg);

    // Keep the progress renderer alive until processing and all updates finish.
    let progress_thread = progress.map(|(file_sz, rx)| {
        std::thread::spawn(move || {
            let pb = utils::init_progress(file_sz);
            for position in rx {
                utils::progress(&pb, position);
            }
            pb
        })
    });

    // Handling the future based on the presence of a timeout
    let result = match timeout_option {
        Some(dur) => match block_on(fut.timeout(dur)) {
            Ok(Ok(_)) => StreamExitType::UnexpectedEofInTuner,
            Ok(Err(e)) => StreamExitType::Error(e),
            _ => StreamExitType::Timeout,
        },
        None => block_on(fut).map_or_else(StreamExitType::Error, StreamExitType::Success),
    };

    // The future's sender has been dropped. Drain pending progress updates
    // before rendering the final byte count and exiting.
    if let Some(thread) = progress_thread {
        let pb = thread.join().expect("Progress rendering thread panicked");
        match &result {
            StreamExitType::Success(bytes) => {
                utils::progress(&pb, *bytes);
                pb.finish();
            }
            _ => pb.abandon(),
        }
    }

    match result {
        StreamExitType::Success(_) => {}
        StreamExitType::Timeout => {}
        StreamExitType::Error(_) => {}
        StreamExitType::UnexpectedEofInTuner => {}
    }
}
