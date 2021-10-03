use clap::App;
use std::time::Duration;
use futures::executor::block_on;

use b25_sys::access_control::types::WorkingKey;
use crate::tuner_base::Tuned;

mod channels;
mod tuner_base;
fn main() {
    let yaml = clap::load_yaml!("arg.yaml");
    let matches = App::from_yaml(yaml).get_matches();

    let device = matches.value_of("device").unwrap();

    //tune
    let chan = matches.value_of("channel-name").unwrap();
    let frequency = channels::Channel::from_ch_str(chan);

    //open a device
    let tuned = match tuner_base::tune(device, frequency)
    {
        Ok(t) => t,
        Err(e) => {
             eprintln!("{}", e);
             return;
        }
    };

    //check S/N rate
    if matches.is_present("checksignal") {
        tuned.signal_quality();
        return;
    }

    //set duration
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

    //open AsyncRead
    let mut source = tuned.open();
    //ARIB-STD-B25 decode
    let r = {
        //ecm
        let key = {
            let emm = matches.is_present("no-card");
            match (matches.value_of("key0"),matches.value_of("key1")) {
                (None, None) => None,
                (Some(k0), Some(k1)) if emm => Some(WorkingKey{0: k0.parse().unwrap(), 1: k1.parse().unwrap() }),
                _ => panic!("Specify both of the keys")
            }
        };
        //TODO:get emm ids from clap
        let ids = Vec::new(); //empty

        StreamDecoder::new(source.as_mut(), key, ids)
    };


    let core_task = async {
        if let Some(filename) = matches.value_of("output") {
            let mut w = AllowStdIo::new(std::fs::File::create(filename).unwrap());
            let (rw, h) = recording(r, &mut w);
            config_timer_handler(rec_dur, h);
            rw.await
        } else {
            let out = std::io::stdout();
            let mut w = AllowStdIo::new(out.lock());
            let (rw, abort_handle) = recording(r, &mut w);
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
use b25_sys::StreamDecoder;

fn recording<R: AsyncRead, W: AsyncWrite + Unpin>(
    from: R,
    to: &mut W,
) -> (CopyBuf<'_, BufReader<R>, W>, AbortHandle) {
    let r = futures::io::BufReader::with_capacity(20000 * 40, from);
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
