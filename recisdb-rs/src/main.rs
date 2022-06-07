#[macro_use]
extern crate cfg_if;

use std::error::Error;
use std::io::Write;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use b25_sys::futures::executor::block_on;
use b25_sys::futures::future::AbortHandle;
use b25_sys::futures::io::{AllowStdIo, BufReader};
use b25_sys::futures::AsyncBufRead;
use clap::Parser;

use crate::context::Commands;
use b25_sys::StreamDecoder;
use b25_sys::WorkingKey;

use crate::tuner_base::Tuned;

mod channels;
mod context;
mod tuner_base;

fn get_src(
    device: Option<String>,
    channel: Option<channels::Channel>,
    source: Option<String>,
) -> Result<Box<dyn AsyncBufRead + Unpin>, Box<dyn Error>> {
    if let Some(src) = device {
        crate::tuner_base::tune(&src, channel.unwrap()).map(|tuned| tuned.open_stream())
    } else if let Some(src) = source {
        let input = BufReader::new(AllowStdIo::new(std::fs::File::open(src)?));
        Ok(Box::new(input) as Box<dyn AsyncBufRead + Unpin>)
    } else {
        panic!("no source specified");
    }
}

fn get_output(directory: Option<String>) -> Result<Box<dyn Write>, std::io::Error> {
    Ok(if let Some(dir) = directory {
        let dir = std::fs::canonicalize(dir)?;
        Box::new(std::fs::File::create(dir)?) as Box<dyn Write>
    } else {
        Box::new(std::io::stdout().lock()) as Box<dyn Write>
    })
}

fn main() {
    let arg = context::Cli::parse();
    println!("{:?}", arg);

    let result = match arg.command {
        Commands::Tune {
            device,
            channel,
            time,
            key0,
            key1,
            source,
            directory,
        } => {
            let key = match (key0, key1) {
                (None, None) => None,
                (Some(k0), Some(k1)) => Some(WorkingKey {
                    0: u64::from_str_radix(k0.trim_start_matches("0x"), 16).unwrap(),
                    1: u64::from_str_radix(k1.trim_start_matches("0x"), 16).unwrap(),
                }),
                _ => panic!("Specify both of the keys"),
            };
            let rec_duration = time.map(Duration::from_secs_f64);
            let mut src = get_src(
                device,
                channel.map(|s| channels::Channel::from_ch_str(s)),
                source,
            )
            .unwrap();
            let from = StreamDecoder::new(&mut src, key);
            let output = &mut b25_sys::futures::io::AllowStdIo::new(get_output(directory).unwrap());
            let (stream, abort_handle) = b25_sys::futures::io::copy_buf_abortable(
                b25_sys::futures::io::BufReader::with_capacity(20000 * 40, from),
                output,
            );

            // Configure sigint trigger
            config_timer_handler(rec_duration, abort_handle);

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
                println!("S/N = {}[dB]\r", tuned.signal_quality());
                if flag2.load(Ordering::Relaxed) {
                    return;
                }
                std::thread::sleep(Duration::from_secs(1));
            }
        }
    };
    match result {
        Ok(Ok(_)) => eprintln!("Stream has gracefully reached its end."),
        Ok(Err(a)) => eprintln!("{}", a),
        Err(e) => eprintln!("{}", e),
    }
    eprintln!("Finished");
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
