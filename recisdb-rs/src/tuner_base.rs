#![allow(unused_imports)]

use std::error::Error;

use futures::AsyncRead;

use crate::channels::Channel;

mod IBonDriver;
mod error;
mod linux;
mod windows;

pub trait Tuned {
    fn signal_quality(&self) -> f64;
    fn set_lnb(&self) -> Result<i8, String>;
    fn open_stream(self) -> Box<dyn AsyncRead + Unpin>;
}

pub fn tune(path: &str, channel: Channel) -> Result<impl Tuned, Box<dyn Error>> {
    use crate::tuner_base::error::GeneralError::EnvCompatFailure;
    cfg_if! {
        if #[cfg(target_os = "linux")]
        {
            use crate::tuner_base::linux::TunedDevice;
            TunedDevice::tune(path, channel, 0)
        }
        else if #[cfg(target_os = "windows")] {
            use crate::tuner_base::windows::TunedDevice;
            TunedDevice::tune(path, channel)
        }
        else { Err((EnvCompatFailure).into()) }
    }
}
