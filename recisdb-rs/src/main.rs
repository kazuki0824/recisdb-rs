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

mod channels;
mod context;
mod utils;

fn main() {
    let arg = context::Cli::parse();
    info!("{:?}", arg);

    utils::initialize_logger();

    let result = match arg.command {
        Commands::Tune {
            device,
            channel,
            time,
            key0,
            key1,
            output,
        } => {
            // Settings
            let settings = {
                DecoderOptions {
                    working_key: utils::parse_keys(key0, key1),
                    round: 4,
                    strip: true,
                    emm: true,
                    simd: false,
                    verbose: false,
                }
            };

            //Recording duration
            let rec_duration = time.map(Duration::from_secs_f64);

            //Combine the source, decoder, and output into a single future
            let mut src = utils::get_src(
                device,
                channel.map(|s| channels::Channel::from_ch_str(s)),
                None,
            )
                .unwrap();
            let from = StreamDecoder::new(&mut src, settings);
            let output = &mut AllowStdIo::new(utils::get_output(output).unwrap());
            let (stream, abort_handle) = futures_util::io::copy_buf_abortable(
                BufReader::with_capacity(20000 * 40, from),
                output,
            );

            // Configure sigint trigger
            config_timer_handler(rec_duration, abort_handle);

            block_on(stream)
        }
        Commands::Decode {
            source, key0, key1, output
        } => {
            // Settings
            let settings = {
                DecoderOptions {
                    working_key: utils::parse_keys(key0, key1),
                    round: 4,
                    strip: true,
                    emm: true,
                    simd: false,
                    verbose: false,
                }
            };

            //Combine the source, decoder, and output into a single future
            let mut src = utils::get_src(
                None,
                None,
                source,
            )
                .unwrap();
            let from = StreamDecoder::new(&mut src, settings);
            let output = &mut AllowStdIo::new(utils::get_output(output).unwrap());
            let (stream, abort_handle) = futures_util::io::copy_buf_abortable(
                BufReader::with_capacity(20000 * 40, from),
                output,
            );

            // Configure sigint trigger
            config_timer_handler(None, abort_handle);

            block_on(stream)
        }
        Commands::Checksignal { device, channel } => {
            //open tuner and tune to channel
            let channel = channel.map(|s| channels::Channel::from_ch_str(s));
            let tuned = crate::tuner_base::tune(&device, channel.unwrap()).unwrap();
            //configure sigint trigger
            let flag = std::sync::Arc::new(AtomicBool::new(false));
            let flag2 = flag.clone();
            ctrlc::set_handler(move || flag.store(true, Ordering::Relaxed)).unwrap();

            loop {
                std::thread::sleep(Duration::from_secs(1));
                if flag2.load(Ordering::Relaxed) {
                    return;
                }
                println!("S/N = {}[dB]\r", tuned.signal_quality());
            }
        }
    };
    match result {
        Ok(Ok(_)) => info!("Stream has gracefully reached its end."),
        Ok(Err(a)) => info!("{}", a),
        Err(e) => error!("{}", e),
    }
    info!("Finished");
}

fn config_timer_handler(duration: Option<Duration>, abort_handle: AbortHandle) {
    //configure timer
    if let Some(record_duration) = duration {
        let h = abort_handle.clone();
        std::thread::spawn(move || {
            std::thread::sleep(record_duration);
            h.abort();
        });
    }
    //configure sigint trigger
    ctrlc::set_handler(move || abort_handle.abort()).unwrap();
}
