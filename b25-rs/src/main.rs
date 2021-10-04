use std::fs::File;
use std::io::{stdin, stdout};
use std::thread::JoinHandle;

use futures::{AsyncRead, AsyncWrite};
use futures::executor::block_on;
use futures::future::AbortHandle;
use futures::io::{AllowStdIo, BufReader, CopyBuf};

use b25_sys::access_control::types::WorkingKey;
use b25_sys::StreamDecoder;

fn main() {
    println!("Hello, world!");


    let yaml = clap::load_yaml!("arg.yaml");
    let matches = clap::App::from_yaml(yaml).get_matches();

    let key = {
        let emm = matches.is_present("no-card");
        match (matches.value_of("key0"),matches.value_of("key1")) {
            (None, None) => None,
            (Some(k0), Some(k1)) if emm => Some(WorkingKey{0: k0.parse().unwrap(), 1: k1.parse().unwrap() }),
            _ => panic!("Specify both of the keys")
        }
    };

    let result = async {
        let standard_in = stdin();
        let standard_out = stdout();
        let input = matches.value_of("input");
        let out = matches.value_of("output");

        match (input, out)
        {
            (Some(i), Some(o)) =>
                {
                    eprintln!("Input: {}", i);
                    eprintln!("Output: {}", o);
                    let (i, o) = (File::open(i).unwrap() , File::create(o).unwrap());
                    let (mut i, mut o) = (AllowStdIo::new(i), AllowStdIo::new(o));
                    let x = recording(&mut i,&mut o, key);
                    config_ctrlc_handler(x.1);
                    x.0.await
                },
            (None, Some(o)) =>
                {
                    eprintln!("stdin is selected");
                    eprintln!("Output: {}", o);
                    let (i, o) = (standard_in.lock() , File::create(o).unwrap());
                    let (mut i, mut o) = (AllowStdIo::new(i), AllowStdIo::new(o));
                    let x = recording(&mut i,&mut o, key);
                    config_ctrlc_handler(x.1);
                    x.0.await
                },
            (Some(i), None) =>
                {
                    eprintln!("Input: {}", i);
                    eprintln!("stdout is selected");
                    let (i, o) = (File::open(i).unwrap() , standard_out.lock());
                    let (mut i, mut o) = (AllowStdIo::new(i), AllowStdIo::new(o));
                    let x = recording(&mut i,&mut o, key);
                    config_ctrlc_handler(x.1);
                    x.0.await
                },
            (None,None) =>
                {
                    eprintln!("stdin is selected");
                    eprintln!("stdout is selected");
                    let (i, o) = (standard_in.lock(),standard_out.lock());
                    let (mut i, mut o) = (AllowStdIo::new(i), AllowStdIo::new(o));
                    let x = recording(&mut i,&mut o, key);
                    config_ctrlc_handler(x.1);
                    x.0.await
                }
        }
    };

    recv_emm();
    let result = block_on(result);
    match result {
        Ok(Ok(n)) => eprintln!("Stream reached its end. {} B received.", n),
        Ok(Err(a)) => eprintln!("{}", a),
        Err(e) => eprintln!("{}", e),
    }
    eprintln!("Finished");

}
fn recording<'a, R: AsyncRead + Unpin, W: AsyncWrite + Unpin>(
    from: &'a mut R,
    to: &'a mut W,
    key: Option<WorkingKey>
) -> (CopyBuf<'a, BufReader<StreamDecoder<'a>>, W>, AbortHandle) {

    let decoder = StreamDecoder::new(from, key, Vec::new());

    let r = futures::io::BufReader::with_capacity(20000 * 40, decoder);
    futures::io::copy_buf(r, to)
}
fn config_ctrlc_handler(abort_handle: AbortHandle)
{
    //configure sigint trigger
    ctrlc::set_handler(move || abort_handle.abort()).unwrap();
}
fn recv_emm() -> JoinHandle<()>
{
    std::thread::spawn(move || {
        if let Some(r) = b25_sys::receive_emm()
        {
            loop {
                if let Ok(res) = r.try_recv()
                {
                    todo!("impl trait Display");
                    //eprintln!("{}", res);
                }
            }
        }
    })
}
