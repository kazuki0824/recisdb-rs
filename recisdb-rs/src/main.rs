use crate::io::StreamCompletion;
use crate::utils::StreamExitType;
use clap::Parser;
use futures_executor::block_on;
use futures_time::future::FutureExt;
use log::info;
use indicatif::ProgressBar;

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
    let progress_thread = progress.map(|(file_sz, rx)| render_progress(file_sz, rx));

    // Handling the future based on the presence of a timeout
    let result = match timeout_option {
        Some(dur) => match block_on(fut.timeout(dur)) {
            Ok(Ok(StreamCompletion::Finished(_))) => StreamExitType::UnexpectedEofInTuner,
            Ok(Ok(StreamCompletion::Interrupted(bytes))) => StreamExitType::Interrupted(bytes),
            Ok(Err(e)) => StreamExitType::Error(e),
            Err(_) => StreamExitType::Timeout,
        },
        None => match block_on(fut) {
            Ok(StreamCompletion::Finished(bytes)) => StreamExitType::Success(bytes),
            Ok(StreamCompletion::Interrupted(bytes)) => StreamExitType::Interrupted(bytes),
            Err(e) => StreamExitType::Error(e),
        },
    };

    // The future's sender has been dropped. Drain pending progress updates
    // before rendering the final byte count and exiting.
    if let Some(thread) = progress_thread {
        let pb = thread.join().expect("Progress rendering thread panicked");
        finish_progress(&pb, &result);
    }

    match result {
        StreamExitType::Success(_) => {}
        StreamExitType::Interrupted(_) => {}
        StreamExitType::Timeout => {}
        StreamExitType::Error(_) => {}
        StreamExitType::UnexpectedEofInTuner => {}
    }
}

fn render_progress(
    file_sz: u64,
    rx: std::sync::mpsc::Receiver<u64>,
) -> std::thread::JoinHandle<ProgressBar> {
    std::thread::spawn(move || {
        let pb = utils::init_progress(file_sz);
        for position in rx {
            utils::progress(&pb, position);
        }
        pb
    })
}

fn finish_progress(pb: &ProgressBar, result: &StreamExitType) {
    match result {
        StreamExitType::Success(bytes) => {
            utils::progress(pb, *bytes);
            pb.finish();
        }
        StreamExitType::Interrupted(bytes) => {
            utils::progress(pb, *bytes);
            pb.abandon();
        }
        _ => pb.abandon(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normal_completion_finishes_progress() {
        let pb = ProgressBar::new(100);
        finish_progress(&pb, &StreamExitType::Success(100));
        assert!(pb.is_finished());
        assert_eq!(pb.position(), 100);
    }

    #[test]
    fn interruption_keeps_partial_progress() {
        let pb = ProgressBar::new(100);
        finish_progress(&pb, &StreamExitType::Interrupted(30));
        assert!(pb.is_finished());
        assert_eq!(pb.position(), 30);
    }

    #[test]
    fn error_does_not_force_completion() {
        let pb = ProgressBar::new(100);
        pb.set_position(30);
        let error = std::io::Error::new(std::io::ErrorKind::Other, "decode failed");
        finish_progress(&pb, &StreamExitType::Error(error));
        assert!(pb.is_finished());
        assert_eq!(pb.position(), 30);
    }

    #[test]
    fn renderer_drains_pending_updates_before_join() {
        let (tx, rx) = std::sync::mpsc::channel();
        let renderer = render_progress(100, rx);
        for position in 0..=100 {
            tx.send(position).unwrap();
        }
        drop(tx);
        let pb = renderer.join().unwrap();
        assert_eq!(pb.position(), 100);
        assert!(!pb.is_finished());
    }
}
