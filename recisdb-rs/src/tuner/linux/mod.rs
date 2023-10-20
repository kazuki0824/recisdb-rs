use crate::channels::Channel;
use crate::tuner::{Tunable, Voltage};
use fancy_regex::Regex;
use futures_util::{AsyncBufRead, AsyncRead};
use std::io::Error;
use std::pin::Pin;
use std::task::{Context, Poll};

mod character_device;
mod dvbv5;

pub enum UnTunedTuner {
    DvbV5(dvbv5::UnTunedTuner),
    Character(character_device::UnTunedTuner),
}
impl UnTunedTuner {
    pub fn new(path: String) -> Result<UnTunedTuner, Error> {
        if let Ok(Some(mat)) = Regex::new(r"[1-9]*[0-9]\|[1-9]*[0-9]").unwrap().find(&path) {
            let result: Vec<u8> = mat
                .as_str()
                .split("|")
                .map(|v| v.parse().unwrap())
                .collect();
            Ok(UnTunedTuner::DvbV5(dvbv5::UnTunedTuner::new(
                result[0], result[1],
            )?))
        } else {
            Ok(UnTunedTuner::Character(
                character_device::UnTunedTuner::new(path)?,
            ))
        }
    }
}

pub enum Tuner {
    DvbV5(dvbv5::Tuner),
    Character(character_device::Tuner),
}

impl Tuner {
    pub fn signal_quality(&self) -> f64 {
        match self {
            Tuner::DvbV5(_) => {
                todo!()
            }
            Tuner::Character(inner) => inner.signal_quality(),
        }
    }
}

impl Tunable for UnTunedTuner {
    fn tune(self, ch: Channel, lnb: Option<Voltage>) -> Result<Tuner, Error> {
        match self {
            UnTunedTuner::DvbV5(inner) => Ok(Tuner::DvbV5(inner.tune(ch, lnb)?)),
            UnTunedTuner::Character(inner) => Ok(Tuner::Character(inner.tune(ch, lnb)?)),
        }
    }
}

impl AsyncRead for Tuner {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<std::io::Result<usize>> {
        match self.get_mut() {
            Tuner::DvbV5(_) => {
                todo!()
            }
            Tuner::Character(inner) => Pin::new(inner).poll_read(cx, buf),
        }
    }
}

impl AsyncBufRead for Tuner {
    fn poll_fill_buf(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<&[u8]>> {
        match self.get_mut() {
            Tuner::DvbV5(inner) => Pin::new(inner).poll_fill_buf(cx),
            Tuner::Character(inner) => Pin::new(inner).poll_fill_buf(cx),
        }
    }

    fn consume(self: Pin<&mut Self>, amt: usize) {
        match self.get_mut() {
            Tuner::DvbV5(inner) => Pin::new(inner).consume(amt),
            Tuner::Character(inner) => Pin::new(inner).consume(amt),
        }
    }
}
