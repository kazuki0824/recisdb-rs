use b25_sys::futures_io::{AsyncBufRead, AsyncRead};
use futures_util::io::BufReader;
use std::error::Error;
use std::mem::ManuallyDrop;
use std::ptr::NonNull;
use std::task::Poll;
use std::time::Duration;
use log::info;

use crate::channels::*;
use crate::tuner_base::error::BonDriverError;
use crate::tuner_base::IBonDriver::{BonDriver, IBon};
use crate::tuner_base::Voltage;

pub struct TunedDevice {
    dll_imported: ManuallyDrop<BonDriver>,
    pub interface: ManuallyDrop<IBon<10000>>,
}

impl TunedDevice {
    fn enum_all_available_space_channels(
        interface: &IBon<10000>,
    ) -> Result<Vec<ChannelSpace>, BonDriverError> {
        let mut channels = Vec::new();
        let mut i = 0;
        while let Some(space) = interface.EnumTuningSpace(i) {
            let mut j = 0;
            while let Some(channel) = interface.EnumChannelName(i, j) {
                channels.push(ChannelSpace {
                    space: i,
                    ch: j,
                    space_description: Some(space.clone()),
                    ch_description: Some(channel.clone()),
                });
                info!("{}-{} {}-{}", i, j, space, channel);
                j += 1;
            }
            i += 1;
        }
        Ok(channels)
    }
    pub(crate) fn tune(path: &str, channel: Channel) -> Result<Self, Box<dyn Error>> {
        let path_canonicalized = std::fs::canonicalize(path)?;
        let dll_imported = unsafe {
            let lib = BonDriver::new(path)?;
            ManuallyDrop::new(lib)
        };
        info!("[BonDriver]{:?} is loaded.", path_canonicalized);
        let interface = {
            let i_bon = dll_imported.create_interface();
            let ver = if i_bon.2.is_none() {
                1
            } else if i_bon.3.is_none() {
                2
            } else {
                3
            };
            info!(
                "[BonDriver] An interface is generated. The version is {}.",
                ver
            );

            ManuallyDrop::new(i_bon)
        };

        interface.OpenTuner()?;
        if let Some(phy_ch) = channel.try_get_physical_num() {
            interface.SetChannel(phy_ch)?;
        } else if let ChannelType::Bon(space) = channel.ch_type {
            let channels = Self::enum_all_available_space_channels(&interface)?;
            interface.SetChannelBySpace(space.space, space.ch)?;
        }

        Ok(TunedDevice {
            dll_imported,
            interface,
        })
    }
}

impl super::Tuned for TunedDevice {
    fn signal_quality(&self) -> f64 {
        todo!()
    }

    fn open_stream(self) -> Box<dyn AsyncBufRead + Unpin> {
        let with_buffer = BufReader::new(self);
        Box::new(with_buffer)
    }
}

impl Drop for TunedDevice {
    fn drop(&mut self) {
        unsafe {
            //NOTE: The drop order should be explicitly defined like below
            ManuallyDrop::drop(&mut self.interface);
            ManuallyDrop::drop(&mut self.dll_imported);
        }
    }
}

impl AsyncRead for TunedDevice {
    fn poll_read(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut [u8],
    ) -> Poll<std::io::Result<usize>> {
        match self.interface.GetTsStream() {
            Ok((recv, remaining)) if !recv.is_empty() => {
                info!("{} bytes received.", recv.len());
                buf[0..recv.len()].copy_from_slice(&recv[0..]);
                Poll::Ready(Ok(buf.len()))
            }
            Ok((recv, remaining)) if recv.is_empty() && remaining > 0 => {
                info!("{} remaining.", remaining);
                cx.waker().wake_by_ref();
                Poll::Pending
            }
            _ => {
                let w = cx.waker().clone();
                //self.interface.WaitTsStream(Duration::from_millis(10));
                std::thread::spawn(move || {
                    std::thread::sleep(Duration::from_millis(100));
                    w.wake();
                });
                Poll::Pending
            }
        }
    }
}
