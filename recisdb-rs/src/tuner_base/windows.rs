use std::error::Error;
use std::mem::ManuallyDrop;
use std::ptr::NonNull;
use std::time::Duration;

use futures::future::poll_fn;
use futures::AsyncRead;

use crate::channels::Channel;
use crate::tuner_base::error::BonDriverError;
use crate::tuner_base::IBonDriver::{BonDriver, IBon};

pub struct TunedDevice {
    dll_imported: ManuallyDrop<BonDriver>,
    pub interface: ManuallyDrop<IBon<10000>>,
}

impl TunedDevice {
    pub(crate) fn tune(path: &str, channel: Channel) -> Result<Self, Box<dyn Error>> {
        let dll_imported = unsafe {
            let lib = BonDriver::new(path)?;
            ManuallyDrop::new(lib)
        };
        eprintln!("[BonDriver]{} is loaded", path);
        let interface = {
            let i_bon = dll_imported.create();
            ManuallyDrop::new(i_bon)
        };
        eprintln!("[BonDriver] Interface generated.");

        interface.OpenTuner()?;
        interface.SetChannel(channel.physical_ch_num)?;

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

    fn open_stream(self) -> Box<dyn AsyncRead + Unpin> {
        use futures::stream::poll_fn;
        use futures::task::Poll;
        use futures::TryStreamExt;
        let stream = poll_fn(move |_| {
            if self.interface.WaitTsStream(Duration::from_millis(1000)) {
                match self.interface.GetTsStream() {
                    Ok((buf, remaining)) => Poll::Ready(Some(Ok(buf))),
                    Err(e) => {
                        //TODO:Convert Error into io::Error?
                        //Poll::Ready(Some(Err(e.into())))
                        Poll::Ready(None)
                    }
                }
            } else {
                Poll::Pending
            }
        });
        Box::new(stream.into_async_read())
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
