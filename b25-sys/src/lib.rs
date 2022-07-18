use futures_core::ready;
use std::cell::Cell;
use std::io::{Read, Write};
use std::pin::Pin;
use std::sync::Mutex;
use std::task::{Context, Poll};

pub use futures_io;
use futures_io::{AsyncBufRead, AsyncRead};
use pin_project_lite::pin_project;

use crate::bindings::InnerDecoder;

#[cfg(feature = "block00cbc")]
mod access_control;
mod bindings;

static KEY0: Mutex<Vec<u64>> = Mutex::new(Vec::new());
static KEY1: Mutex<Vec<u64>> = Mutex::new(Vec::new());

pub fn set_keys(key0: Vec<u64>, key1: Vec<u64>) {
    KEY0.lock().unwrap().clear();
    KEY0.lock().unwrap().extend(key0);
    KEY1.lock().unwrap().clear();
    KEY1.lock().unwrap().extend(key1);
}

pin_project! {
    pub struct StreamDecoder<'a> {
        #[pin]
        reader: &'a mut (dyn AsyncBufRead + Unpin),
        received: Cell<usize>,
        sent: Cell<usize>,
        inner: InnerDecoder,
    }
    impl PinnedDrop for StreamDecoder<'_> {
        fn drop(this: Pin<&mut Self>) {
            eprintln!("{}B received, and {}B converted.", this.received.get(), this.sent.get());
        }
    }
}

pub struct DecoderOptions {
    pub enable_working_key: bool,
    pub round: i32,
    pub strip: bool,
    pub emm: bool,
    pub simd: bool,
    pub verbose: bool,
}

impl<'a> StreamDecoder<'a> {
    pub fn new(reader: &'a mut (dyn AsyncBufRead + Unpin), opt: DecoderOptions) -> Self {
        let inner = unsafe {
            let inner = InnerDecoder::new(opt.enable_working_key).unwrap();
            // Set options to the decoder
            inner.dec.as_ref().set_multi2_round(opt.round);
            inner.dec.as_ref().set_strip(if opt.strip { 1 } else { 0 });
            inner.dec.as_ref().set_emm_proc(if opt.emm { 1 } else { 0 });
            inner
                .dec
                .as_ref()
                .set_simd_mode(if opt.simd { 1 } else { 0 });

            // TODO: Verbose mode and power control is not implemented yet.
            inner
        };

        Self {
            received: Cell::new(0),
            sent: Cell::new(0),
            reader,
            inner,
        }
    }
}

impl AsyncRead for StreamDecoder<'_> {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<std::io::Result<usize>> {
        let mut this = self.project();

        //try receiving
        let recv = ready!(this.reader.as_mut().poll_fill_buf(cx))?;
        //get n
        let n = recv.len();
        //if 0, exit, or continue waiting for next
        if n == 0 {
            this.reader.as_mut().consume(0);
            return Poll::Ready(Ok(0));
            // cx.waker().wake_by_ref();
            // Poll::Pending
        } else {
            //Write to this.inner in order to decode, and read from this.inner in order to write to buf
            this.inner.write_all(recv).expect("write_all failed");
            this.reader.as_mut().consume(n);
            this.received.set(this.received.get() + n);

            //try reading
            let read = this.inner.read(buf);
            //if 0, exit, or continue waiting for next
            match read {
                Ok(0) => {
                    cx.waker().wake_by_ref();
                    Poll::Pending
                }
                Ok(n) => {
                    this.sent.set(this.sent.get() + n);
                    Poll::Ready(Ok(n))
                }
                Err(e) => Poll::Ready(Err(e)),
            }
        }
    }
}
