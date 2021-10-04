use crate::channels::Channel;
use crate::tuner_base::IBonDriver::{BonDriver, IBon};
use crate::tuner_base::error::BonDriverError;
use std::error::Error;
use std::mem::ManuallyDrop;
use std::ptr::NonNull;
use std::time::Duration;
use futures::AsyncRead;
use futures::future::poll_fn;

pub struct TunedDevice {
    bon_driver_path: String,
    dll_imported: ManuallyDrop<BonDriver>,
    pub interface: ManuallyDrop<IBon<10000>>
}

impl TunedDevice {
    pub(crate) fn tune(path: &str, channel: Channel) -> Result<Self, Box<dyn Error>> {
        let dll_imported = unsafe { 
            let lib = BonDriver::new(path)?;
            ManuallyDrop::new(lib)
        };
        let interface = {
            let i_bon =  dll_imported.create();
            ManuallyDrop::new(i_bon)
        };

        interface.OpenTuner()?;
        interface.SetChannel(channel.physical_ch_num)?;

        Ok(TunedDevice {
            bon_driver_path: path.to_string(),
            dll_imported,
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

    fn open(self) -> Box<dyn AsyncRead + Unpin> {
        use futures::stream::poll_fn;
        use futures::TryStreamExt;
        use futures::task::Poll;
        let stream = poll_fn(move |_| {
            if self.interface.WaitTsStream(Duration::from_millis(1000))
            {                
                match self.interface.GetTsStream()
                {
                    Ok((buf, remaining)) => {
                        Poll::Ready(Some(Ok(buf)))
                    },
                    Err(e) => Poll::Ready(None) //TODO:Poll::Ready(Some(Err(e.into())))
                }
            }
            else {
                Poll::Pending
            }
        });
        Box::new(stream.into_async_read())
    }
}


impl Drop for TunedDevice
{
    fn drop(&mut self) {
        unsafe {
            //NOTE: The drop order should be explicitly defined like below
            ManuallyDrop::drop(&mut self.interface);
            ManuallyDrop::drop(&mut self.dll_imported);
        }
    }
}