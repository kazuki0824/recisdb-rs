use futures_time::time::Duration;
use std::future::Future;
use std::io::Write;

use log::{error, info};

use b25_sys::DecoderOptions;

use crate::channels::{Channel, ChannelType};
use crate::commands::utils::parse_keys;
use crate::context::{Cli, Commands};
use crate::io::AsyncInOutTriple;
use crate::tuner::{Tunable, UnTunedTuner};

pub(crate) mod utils;

/// The behavior the user requested are returned.
/// If an error occurred during preparation, the program bails out with expect().
pub(crate) fn process_command(
    args: Cli,
) -> (
    impl Future<Output = std::io::Result<u64>>,
    Option<Duration>,
    Option<(u64, std::sync::mpsc::Receiver<u64>)>,
) {
    match args.command {
        Commands::Checksignal {
            channel,
            device,
            tsid,
            lnb,
        } => {
            // Get channel
            let channel = channel.map(|ch| Channel::new(ch, tsid)).unwrap();
            if let ChannelType::Undefined = channel.ch_type {
                error!("The specified channel is invalid.");
                std::process::exit(1);
            }
            info!("Tuner: {}", device);
            info!(
                "Channel: {} / {}",
                channel.get_raw_ch_name(),
                channel.ch_type
            );

            // Open tuner and tune to channel
            let tuned = match UnTunedTuner::new(device)
                .map_err(|e| utils::error_handler::handle_opening_error(e.into()))
                .unwrap()
                .tune(channel, lnb)
            {
                Ok(inner) => inner,
                Err(e) => utils::error_handler::handle_tuning_error(e),
            };

            // ctrlc::set_handler(|| std::process::exit(0)).expect("Error setting Ctrl-C handler");

            loop {
                print!("{:.2}dB\r", tuned.signal_quality());
                std::io::stdout().flush().unwrap();
                std::thread::sleep(Duration::from_secs_f64(1.0).into())
            }
        }
        Commands::Tune {
            device,
            channel,
            tsid,
            time,
            no_decode: disable_decode,
            lnb,
            key0,
            key1,
            no_simd,
            no_strip,
            output,
            continue_on_error,
        } => {
            // Get channel
            let channel = channel.map(|ch| Channel::new(ch, tsid)).unwrap();
            if let ChannelType::Undefined = channel.ch_type {
                error!("The specified channel is invalid.");
                std::process::exit(1);
            }
            info!("Tuner: {}", device.clone().unwrap());
            info!(
                "Channel: {} / {}",
                channel.get_raw_ch_name(),
                channel.ch_type
            );

            // Recording duration
            let rec_duration = time.map(Duration::from_secs_f64);
            match rec_duration {
                Some(duration) => {
                    info!("Recording duration: {} seconds", duration.as_secs_f64());
                }
                None => {
                    info!("Recording duration: Infinite");
                }
            }

            // in, out, dec
            let (input, _) = utils::get_src(device, Some(channel), None, lnb)
                .map_err(|e| {
                    error!("Failed to open input source: {}", e);
                    std::process::exit(1);
                })
                .unwrap();
            let output = utils::get_output(output)
                .map_err(|e| {
                    error!("Failed to open output: {}", e.kind());
                    std::process::exit(1);
                })
                .unwrap();
            let dec = if disable_decode {
                info!("Decode: Disabled");
                None
            } else {
                info!("Decode: Enabled");
                Some(DecoderOptions {
                    enable_working_key: parse_keys(key0, key1),
                    simd: !no_simd,
                    strip: !no_strip,
                    ..DecoderOptions::default()
                })
            };

            let (body, _) = AsyncInOutTriple::new(input, output, dec, continue_on_error);
            info!("Recording...");
            (body, rec_duration, None)
        }
        Commands::Decode {
            source,
            key0,
            key1,
            no_simd,
            no_strip,
            output,
        } => {
            // in, out, dec
            let (input, input_sz) = utils::get_src(None, None, source, None)
                .map_err(|e| {
                    error!("Failed to open input source: {}", e);
                    std::process::exit(1);
                })
                .unwrap();
            let output = utils::get_output(output)
                .map_err(|e| {
                    error!("Failed to open output: {}", e.kind());
                    std::process::exit(1);
                })
                .unwrap();
            let dec = Some(DecoderOptions {
                enable_working_key: parse_keys(key0, key1),
                simd: !no_simd,
                strip: !no_strip,
                ..DecoderOptions::default()
            });

            let (body, progress) = AsyncInOutTriple::new(input, output, dec, false);
            info!("Decoding...");
            (body, None, input_sz.map(|sz| (sz, progress)))
        }
    }
}
