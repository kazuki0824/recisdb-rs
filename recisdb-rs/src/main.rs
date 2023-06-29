#[macro_use]
extern crate cfg_if;

use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use clap::Parser;
use futures_executor::block_on;
use futures_util::future::AbortHandle;
use futures_util::io::{AllowStdIo, BufReader};
use log::{error, info};

use b25_sys::{DecoderOptions, StreamDecoder};

use crate::context::Commands;
use crate::tuner_base::Tuned;

mod channels;
mod context;
mod tuner_base;
mod utils;

fn handle_tuning_error(e: Box<dyn std::error::Error>) -> ! {
    if let Some(nix_err) = e.downcast_ref::<nix::Error>() {
        let current_errno = nix::errno::Errno::from_i32(nix::errno::errno());
        match current_errno {
            nix::errno::Errno::EAGAIN => {
                error!("Channel selection failed. The channel may not be received.");
            },
            nix::errno::Errno::EINVAL => {
                error!("The specified channel is invalid.");
            },
            _ => {
                error!("Unexpected Linux error: {}", nix_err);
            }
        }
    } else if let Some(io_error) = e.downcast_ref::<std::io::Error>() {
        if let Some(raw_os_error) = io_error.raw_os_error() {
            match raw_os_error {
                libc::EALREADY => {
                    error!("The tuner device is already in use.");
                },
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

fn main() {
    let arg = context::Cli::parse();
    info!("{:?}", arg);

    utils::initialize_logger();

    let result = match arg.command {
        Commands::Tune {
            device,
            channel,
            time,
            output,
            lnb,
            key0,
            key1,
        } => {
            // Settings
            let settings = {
                DecoderOptions {
                    enable_working_key: utils::parse_keys(key0, key1),
                    round: 4,
                    strip: true,
                    emm: true,
                    simd: false,
                    verbose: false,
                }
            };

            // Recording duration
            let rec_duration = time.map(Duration::from_secs_f64);

            // Combine the source, decoder, and output into a single future
            let channel = channel.map(channels::Channel::from_ch_str);
            let channel_clone = channel.clone().unwrap();
            if channel_clone.ch_type == channels::ChannelType::Undefined {
                error!("The specified channel is invalid.");
                std::process::exit(1);
            }
            info!("Channel: {} / {:?}", channel_clone.raw_string, channel_clone.ch_type);
            let mut src = match utils::get_src(
                device,
                channel,
                None,
                lnb,
            ) {
                Ok(src) => src,
                Err(e) => handle_tuning_error(e),
            };
            let from = StreamDecoder::new(&mut src, settings);
            let output = &mut AllowStdIo::new(utils::get_output(output).unwrap());
            let (stream, abort_handle) = futures_util::io::copy_buf_abortable(
                BufReader::with_capacity(20000 * 40, from),
                output,
            );

            // Configure sigint trigger
            config_timer_handler(rec_duration, abort_handle);

            info!("Recording...");
            block_on(stream)
        }
        Commands::Decode {
            source,
            key0,
            key1,
            output,
        } => {
            // Settings
            let settings = {
                DecoderOptions {
                    enable_working_key: utils::parse_keys(key0, key1),
                    round: 4,
                    strip: true,
                    emm: true,
                    simd: false,
                    verbose: false,
                }
            };

            // Combine the source, decoder, and output into a single future
            let mut src = utils::get_src(None, None, source, None).unwrap();
            let from = StreamDecoder::new(&mut src, settings);
            let output = &mut AllowStdIo::new(utils::get_output(output).unwrap());
            let (stream, abort_handle) = futures_util::io::copy_buf_abortable(
                BufReader::with_capacity(20000 * 40, from),
                output,
            );

            // Configure sigint trigger
            config_timer_handler(None, abort_handle);

            info!("Decoding...");
            block_on(stream)
        }
        Commands::Checksignal { device, channel } => {
            // Open tuner and tune to channel
            let channel = channel.map(channels::Channel::from_ch_str).unwrap();
            if channel.ch_type == channels::ChannelType::Undefined {
                error!("The specified channel is invalid.");
                std::process::exit(1);
            }
            info!("Channel: {} / {:?}", channel.raw_string, channel.ch_type);
            let tuned = match crate::tuner_base::tune(&device, channel, None) {
                Ok(tuned) => tuned,
                Err(e) => handle_tuning_error(e),
            };
            // Configure sigint trigger
            let flag = std::sync::Arc::new(AtomicBool::new(false));
            let flag2 = flag.clone();
            ctrlc::set_handler(move || flag.store(true, Ordering::Relaxed)).unwrap();

            loop {
                std::thread::sleep(Duration::from_secs(1));
                if flag2.load(Ordering::Relaxed) {
                    return;
                }
                info!("S/N = {}[dB]\r", tuned.signal_quality());
            }
        }
    };
    match result {
        Ok(Ok(_)) => info!("Stream has gracefully reached its end."),
        Ok(Err(a)) => info!("{}", a),
        Err(e) => error!("{}", e),
    }
    info!("Finished.");
}

fn config_timer_handler(duration: Option<Duration>, abort_handle: AbortHandle) {
    // Configure timer
    if let Some(record_duration) = duration {
        let h = abort_handle.clone();
        std::thread::spawn(move || {
            std::thread::sleep(record_duration);
            h.abort();
        });
    }
    // Configure sigint trigger
    ctrlc::set_handler(move || abort_handle.abort()).unwrap();
}
