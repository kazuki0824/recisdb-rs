use crate::channels::Channel;
use crate::tuner_base::IBonDriver::BonDriver;
use crate::tuner_base::error::BonDriverError;
use std::error::Error;
use std::ptr::NonNull;
use futures::AsyncRead;

impl super::UnTuned for super::Device {
    fn open(path: &str) -> Result<super::Device, Box<dyn Error>> {
        let lib = unsafe { BonDriver::new
            (path) }?;
        let interface = lib.create();
        
        interface.OpenTuner()?;

        Ok(super::Device {
            bon_driver_path: path.to_string(),
            dll_imported: lib,
            kind: super::DeviceKind::WinBon,
            interface
        })
    }
    
    fn tune(mut self, channel: Channel, offset_k_hz: i32) -> Result<super::TunedDevice, Box<dyn Error>> {
        self.interface.SetChannel(channel.physical_ch_num)?;
        Ok(super::TunedDevice { d: self, channel })
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
