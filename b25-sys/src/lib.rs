use log::info;
use std::cell::Cell;
use std::io::{Error, Read, Write};
use std::sync::Mutex;

use crate::bindings::InnerDecoder;

#[cfg(feature = "block00cbc")]
mod access_control;
mod bindings;

static KEY0: Mutex<Vec<u64>> = Mutex::new(Vec::new());
static KEY1: Mutex<Vec<u64>> = Mutex::new(Vec::new());

/// Set keys so that ECM/EMM could be interpreted by StreamDecoder.
pub fn set_keys(key0: Vec<u64>, key1: Vec<u64>) {
    KEY0.lock().unwrap().clear();
    KEY0.lock().unwrap().extend(key0);
    KEY1.lock().unwrap().clear();
    KEY1.lock().unwrap().extend(key1);
}

/// Decode ARIB-STD-B25 stream with libaribb25. Both `Read` and `Write` are implemented.
pub struct StreamDecoder {
    received: Cell<usize>,
    sent: Cell<usize>,
    inner: Mutex<InnerDecoder>,
}
impl Drop for StreamDecoder {
    fn drop(&mut self) {
        info!(
            "{}B received, and {}B converted.",
            self.received.get(),
            self.sent.get()
        );
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

impl Default for DecoderOptions {
    fn default() -> Self {
        Self {
            enable_working_key: false,
            round: 4,
            strip: true,
            emm: true,
            simd: false,
            verbose: false,
        }
    }
}

impl StreamDecoder {
    pub fn new(opt: DecoderOptions) -> Result<Self, Error> {
        let inner = unsafe {
            let inner = InnerDecoder::new(opt.enable_working_key)?;
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

        Ok(Self {
            received: Cell::new(0),
            sent: Cell::new(0),
            inner: Mutex::new(inner),
        })
    }
}

impl Read for StreamDecoder {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.inner.lock().unwrap().read(buf)
    }
}

impl Write for StreamDecoder {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.inner.lock().unwrap().write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.inner.lock().unwrap().flush()
    }
}
