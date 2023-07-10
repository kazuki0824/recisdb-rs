use std::default;
use std::future::Future;
use std::time::Duration;

use log::{error, info};

use b25_sys::DecoderOptions;

use crate::channels::{Channel, ChannelType};
use crate::commands::utils::parse_keys;
use crate::context::{Cli, Commands};
use crate::io::AsyncInOutTriple;

pub(crate) mod utils;

/// The behavior the user requested are returned.
/// If an error occurred during preparation, the program bails out with expect().
pub(crate) fn process_command(args: Cli) -> impl Future<Output = std::io::Result<u64>> {
    match args.command {
        Commands::Checksignal { channel, device } => {
            // Open tuner and tune to channel
            let channel = channel.map(Channel::from_ch_str).unwrap();
            if let ChannelType::Undefined = channel.ch_type {
                error!("The specified channel is invalid.");
                std::process::exit(1);
            }
            info!("Tuner: {}", device);
            info!("{}", channel);

            // let tuned = match crate::tuner_base::tune(&device, channel, None) {
            //     Ok(tuned) => tuned,
            //     Err(e) => //handle_tuning_error(e),
            // };

            todo!("exit program by SIGINT");
        }
        Commands::Tune {
            device,
            channel,
            time,
            disable_decode,
            lnb,
            key0,
            key1,
            output,
        } => {
            // Recording duration
            let rec_duration = time.map(Duration::from_secs_f64);
            // Get channel
            let channel = Channel::from_ch_str(channel.expect("Specify channel correctly"));

            // Emit output
            info!("Tuner: {}", device.clone().unwrap());
            info!("{}", channel);
            let dec = if disable_decode {
                info!("Decode: Disabled");
                None
            } else {
                info!("Decode: Enabled");
                Some(DecoderOptions {
                    enable_working_key: parse_keys(key0, key1),
                    ..DecoderOptions::default()
                })
            };
            match rec_duration {
                Some(duration) => {
                    info!("Recording duration: {} seconds", duration.as_secs_f64());
                }
                None => {
                    info!("Recording duration: Infinite");
                }
            }

            // in, out, dec
            let input = utils::get_src(device, Some(channel), None, lnb)
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

            AsyncInOutTriple::new(input, output, dec)
        }
        Commands::Decode {
            source,
            key0,
            key1,
            output,
        } => {
            // in, out, dec
            let input = utils::get_src(None, None, source, None)
                .map_err(|e| {
                    error!("Failed to open source file: {}", e);
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
                ..DecoderOptions::default()
            });

            AsyncInOutTriple::new(input, output, dec)
        }
    }
}
