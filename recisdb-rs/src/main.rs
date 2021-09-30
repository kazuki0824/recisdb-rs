use crate::tuner_base::{Tuned, UnTuned};
use clap::App;
use std::time::Duration;

mod RecContext;
mod channels;
mod tuner_base;
mod IBonDriver;

use futures::executor::block_on;

fn main() {
    let yaml = clap::load_yaml!("arg.yaml");
    let matches = App::from_yaml(yaml).get_matches();

    let device = matches.value_of("device").unwrap();
    //open a device
    let device = tuner_base::Device::open(device).unwrap();

    //tune
    let chan = matches.value_of("channel-name").unwrap();
    let frequency = channels::Channel::from_ch_str(chan);
    let tuned = device.tune(frequency, 0);

    //check S/N rate
    if matches.is_present("checksignal") {
        tuned.signal_quality();
        return;
    }

    let rec_dur ={
        let time_sec_parsed = matches
            .value_of("time")
            .and_then(|v| v.trim().parse::<f64>().ok());
        match time_sec_parsed {
            Some(record_duration) if record_duration > 0.0 => {
                Some(Duration::from_secs_f64(record_duration))
            }
            _ => None,
        }
    };

    //ARIB-STD-B25 decode
    let config = RecContext::RecConf {
        infinite: rec_dur.is_none(),
        no_card: matches.is_present("no-card"),
    };

    //init decoder
    let context = RecContext::RecContext::new(tuned, config);

    let core_task = async {
        if let Some(filename) = matches.value_of("output") {
            let mut w = AllowStdIo::new(std::fs::File::create(filename).unwrap());
            let (rw, h) = recording(context.decoder, &mut w);
            config_timer_handler(rec_dur, h);
            rw.await
        } else {
            let out = std::io::stdout();
            let mut w = AllowStdIo::new(out.lock());
            let (rw, abort_handle) = recording(context.decoder, &mut w);
            config_timer_handler(rec_dur, abort_handle);
            rw.await
        }
    };

    let result = block_on(core_task);

    match result {
        Ok(Ok(n)) => eprintln!("Stream reached its end. {} B received.", n),
        Ok(Err(a)) => eprintln!("{}", a),
        Err(e) => eprintln!("{}", e),
        //Err(_e) => eprintln!("Tasks finished because of time or sigint."),
    }
    eprintln!("Finished");
}

use futures::io::{AllowStdIo, AsyncRead, AsyncWrite, BufReader, CopyBuf};
use futures::future::AbortHandle;

fn recording<R: AsyncRead, W: AsyncWrite + Unpin>(
    from: R,
    to: &mut W,
) -> (CopyBuf<'_, BufReader<R>, W>, AbortHandle) {
    let r = futures::io::BufReader::with_capacity(RecContext::READ_BUF_SZ * 40, from);
    futures::io::copy_buf(r, to)
}

fn config_timer_handler(duration: Option<Duration>, abort_handle: AbortHandle)
{
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
