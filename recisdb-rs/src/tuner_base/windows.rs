use std::error::Error;
use std::mem::ManuallyDrop;
use std::ptr::NonNull;
use std::time::Duration;

use futures::io::{AsyncBufRead, AsyncRead};

use crate::channels::Channel;
use crate::tuner_base::error::BonDriverError;
use crate::tuner_base::IBonDriver::{BonDriver, IBon};

pub struct TunedDevice {
    dll_imported: ManuallyDrop<BonDriver>,
    pub interface: ManuallyDrop<IBon<10000>>,
}

impl TunedDevice {
    pub(crate) fn tune(path: &str, channel: Channel) -> Result<Self, Box<dyn Error>> {
        let path_canonicalized = std::fs::canonicalize(path)?;
        let dll_imported = unsafe {
            let lib = BonDriver::new(path)?;
            ManuallyDrop::new(lib)
        };
        eprintln!("[BonDriver]{:?} is loaded", path_canonicalized);
        let interface = {
            let i_bon = dll_imported.create_interface();
            let ver = if i_bon.2.is_none() {
                1
            } else if i_bon.3.is_none() {
                2
            } else {
                3
            };
            eprintln!(
                "[BonDriver] An interface is generated. The version is {}.",
                ver
            );

            ManuallyDrop::new(i_bon)
        };

        interface.OpenTuner()?;
        interface.SetChannel(channel)?;

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

    fn set_lnb(&self) -> Result<i8, String> {
        todo!()
    }

    fn open_stream(self) -> Box<dyn AsyncBufRead + Unpin> {
        use futures::io::BufReader;

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

impl AsyncRead for TunedDevice
{
    fn poll_read(self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>, buf: &mut [u8]) -> std::task::Poll<std::io::Result<usize>> {
        use futures::task::Poll;
        if self.interface.WaitTsStream(Duration::from_millis(1000)) {
            match self.interface.GetTsStream() {
                Ok((recv, remaining)) if recv.len() > 0 => {
                    eprintln!("{} bytes recv", recv.len());
                    &buf[0..recv.len()].copy_from_slice(&recv[0..]);
                    Poll::Ready(Ok(buf.len()))
                },
                Err(e) => {
                    //TODO: Convert Error into io::Error?
                    //Poll::Ready(Some(Err(e.into())))
                    Poll::Ready(Ok(0))
                }
                _ => {
                    cx.waker().wake_by_ref();
                    Poll::Pending
                }
            }
        } else {
            cx.waker().wake_by_ref();
            Poll::Pending
        }
    }
}