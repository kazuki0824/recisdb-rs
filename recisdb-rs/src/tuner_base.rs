#![allow(unused_imports)]

use std::error::Error;
use b25_sys::futures_io::AsyncBufRead;

use crate::channels::Channel;

mod IBonDriver;
mod error;
#[cfg(target_os = "linux")]
mod linux;
mod windows;

#[derive(Debug, Clone, clap::ArgEnum)]
pub enum Voltage {
    High11v,
    High15v,
    Low
}

pub trait Tuned {
    fn signal_quality(&self) -> f64;
    fn open_stream(self) -> Box<dyn AsyncBufRead + Unpin>;
}

pub fn tune(path: &str, channel: Channel, voltage: Option<Voltage>) -> Result<impl Tuned, Box<dyn Error>> {
    use crate::tuner_base::error::GeneralError::EnvCompatFailure;
    println!("{:?}", channel);
    cfg_if! {
        if #[cfg(target_os = "linux")]
        {
            use crate::tuner_base::linux::TunedDevice;
            TunedDevice::tune(path, channel, 0, voltage)
        }
        else if #[cfg(target_os = "windows")]
        {
            use crate::tuner_base::windows::TunedDevice;
            TunedDevice::tune(path, channel)
        }
        else { Err((EnvCompatFailure).into()) }
    }
}
