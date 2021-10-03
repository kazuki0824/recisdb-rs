#![allow(unused_imports)]
use crate::channels::Channel;
use crate::tuner_base::IBonDriver::{BonDriver, IBon};
use std::error::Error;
use futures::AsyncRead;

mod error;
#[cfg(target_os = "windows")]
mod IBonDriver;
#[cfg(target_os = "windows")]
mod Win_Bon;
#[cfg(target_os = "linux")]
mod linux;

pub enum DeviceKind {
    LinuxChardev,
    WinBon,
}

pub trait UnTuned {
    fn open(path: &str) -> Result<Device, Box<dyn Error>>;
    fn tune(self, channel: Channel, offset_k_hz: i32) -> Result<TunedDevice, Box<dyn Error>>;
}
pub trait Tuned {
    fn signal_quality(&self) -> f64;
    fn set_lnb(&self) -> Result<i8, String>;
    fn open(&self) -> Box<dyn AsyncRead + Unpin>;
}


//TODO: change opaque TunedDevice type to dyn Tuned and move them into linux / windows, and remove cfg and super::
#[cfg(target_os = "linux")]
pub struct Device {
    pub handle: std::os::unix::io::RawFd,
    kind: DeviceKind,
}
#[cfg(target_os = "linux")]
pub struct TunedDevice {
    pub d: Device,
    pub channel: Channel,
}

#[cfg(target_os = "windows")]
pub struct Device {
    bon_driver_path: String,
    dll_imported: BonDriver,
    pub kind: DeviceKind,
    pub interface: IBon<10000>
}
#[cfg(target_os = "windows")]
pub struct TunedDevice {
    pub d: Device,
    pub channel: Channel,
}
