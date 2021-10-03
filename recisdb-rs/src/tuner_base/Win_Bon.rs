use crate::channels::Channel;
use crate::tuner_base::IBonDriver::{BonDriver, IBon};
use crate::tuner_base::error::BonDriverError;
use std::error::Error;
use std::ptr::NonNull;
use futures::AsyncRead;
use crate::tuner_base::Tuned;

pub struct TunedDevice {
    bon_driver_path: String,
    dll_imported: BonDriver,
    pub interface: IBon<10000>
}

impl TunedDevice {
    pub(crate) fn tune(path: &str, channel: Channel) -> Result<impl Tuned, Box<dyn Error>> {
        let lib = unsafe { BonDriver::new(path) }?;
        let interface = lib.create();

        interface.OpenTuner()?;
        interface.SetChannel(channel.physical_ch_num)?;

        Ok(TunedDevice {
            bon_driver_path: path.to_string(),
            dll_imported: lib,
            interface
        })
    }
}

impl super::Tuned for TunedDevice {
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
