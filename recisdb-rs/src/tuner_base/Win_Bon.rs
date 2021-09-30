use crate::channels::Channel;
use crate::tuner_base::IBonDriver::BonDriver;
use crate::tuner_base::error::BonDriverError;
use std::error::Error;
use std::ptr::NonNull;
use futures::AsyncRead;

impl super::UnTuned for super::Device {
    fn open(path: &str) -> Result<super::Device, Box<dyn Error>> {
        let mut lib = unsafe { BonDriver::new
            (path) }?;
        let mut interface = lib.CreateBonDriver();
        if unsafe {interface.0.as_mut().OpenTuner() == 1} {
            Ok(super::Device {
                bon_driver_path: path.to_string(),
                dll_imported: lib,
                kind: super::DeviceKind::WinBon,
                interface
            })
        }
        else {
            Err(BonDriverError::OpenError.into())
        }
    }
    
    fn tune(mut self, channel: Channel, offset_k_hz: i32) -> super::TunedDevice {
        //TODO:
        unsafe {self.interface.0.as_mut().SetChannel(channel.physical_ch_num)} ;
        super::TunedDevice { d: self, channel }
    }
}

impl super::Tuned for super::TunedDevice {
    fn signal_quality(&self) -> f64 {
        todo!()
    }

    fn set_lnb(&self) -> Result<i8, String> {
        todo!()
    }

    fn open(&self) -> Box<dyn AsyncRead + Unpin> {
        todo!("wrap GetTs into AsyncRead")
    }
}
