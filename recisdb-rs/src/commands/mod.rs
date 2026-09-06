use futures_time::time::Duration;
use std::future::Future;
#[cfg(any(target_os = "linux", target_os = "windows"))]
use std::io::Write;

#[cfg(any(
    target_os = "linux",
    target_os = "windows",
    not(feature = "prioritized_card_reader")
))]
use log::warn;
use log::{error, info};

use b25_sys::DecoderOptions;

#[cfg(any(target_os = "linux", target_os = "windows"))]
use crate::channels::representation::TsFilter;
#[cfg(any(target_os = "linux", target_os = "windows"))]
use crate::channels::{Channel, ChannelType};
use crate::commands::utils::parse_keys;
use crate::context::{Cli, Commands};
use crate::io::AsyncInOutTriple;
#[cfg(any(target_os = "linux", target_os = "windows"))]
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
    #[cfg(any(target_os = "linux", target_os = "windows"))]
    const INPUT_BUF_DEFAULT: usize = 200000;
    #[cfg(any(target_os = "linux", target_os = "windows"))]
    let buf_sz = std::env::var("RECISDB_INPUT_BUF_BYTES")
        .unwrap_or("".to_string())
        .parse()
        .unwrap_or(INPUT_BUF_DEFAULT);

    match args.command {
        #[cfg(any(target_os = "linux", target_os = "windows"))]
        Commands::Checksignal {
            channel,
            device,
            lnb,
        } => {
            // Get channel
            let channel = channel.map(|ch| Channel::new(ch, None)).unwrap();
            if let ChannelType::BS(_, TsFilter::RelTsNum(num)) = channel.ch_type {
                warn!("The specified relative TS num '_{}' has no effect.", num)
            }
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
            let tuned = match UnTunedTuner::new(device, 0)
                .map_err(|e| utils::error_handler::handle_opening_error(e.into()))
                .unwrap()
                .tune(channel, lnb)
            {
                Ok(inner) => inner,
                Err(e) => utils::error_handler::handle_tuning_error(e),
            };

            // ctrlc::set_handler(|| std::process::exit(0)).expect("Error setting Ctrl-C handler");

            loop {
                print!("\r{:.2}dB", tuned.signal_quality());
                std::io::stdout().flush().unwrap();
                std::thread::sleep(Duration::from_secs_f64(1.0).into())
            }
        }
        #[cfg(any(target_os = "linux", target_os = "windows"))]
        Commands::Tune {
            device,
            channel,
            card,
            tsid,
            time,
            no_decode: disable_decode,
            lnb,
            key0,
            key1,
            no_simd,
            no_strip,
            output,
            exit_on_card_error,
        } => {
            // Card reader
            if let Some(name) = card {
                #[cfg(not(feature = "prioritized_card_reader"))]
                warn!("--card {name} has no effect. Use `prioritized_card_reader` feature flag.");

                #[cfg(feature = "prioritized_card_reader")]
                b25_sys::set_card_reader_name(&name);
            }

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
            let input = utils::get_tuner_src(device.unwrap(), channel, lnb, buf_sz)
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

            let (body, _) = AsyncInOutTriple::new(input, output, dec, !exit_on_card_error);
            info!("Recording...");
            (body, rec_duration, None)
        }
        Commands::Decode {
            source,
            card,
            key0,
            key1,
            no_simd,
            no_strip,
            output,
        } => {
            // Card reader
            if let Some(name) = card {
                #[cfg(not(feature = "prioritized_card_reader"))]
                warn!("--card {name} has no effect. Use `prioritized_card_reader` feature flag.");

                #[cfg(feature = "prioritized_card_reader")]
                b25_sys::set_card_reader_name(&name);
            }

            // in, out, dec
            let (input, input_sz) = utils::get_file_src(source)
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
        #[cfg(windows)]
        Commands::Enumerate { device, space } => {
            // Open tuner
            let untuned = UnTunedTuner::new(device, buf_sz)
                .map_err(|e| utils::error_handler::handle_opening_error(e.into()))
                .unwrap();
            if let Some(spacename_channels) = untuned.enum_channels(space) {
                for item in spacename_channels {
                    println!("{}", item)
                }
                std::process::exit(0)
            } else {
                std::process::exit(1)
            }
        }
    }
}
