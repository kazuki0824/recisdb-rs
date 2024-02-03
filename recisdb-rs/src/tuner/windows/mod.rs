use std::io;
use std::io::ErrorKind;
use std::mem::ManuallyDrop;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::Duration;

use futures_util::io::BufReader;
use futures_util::{AsyncBufRead, AsyncRead};
use log::{debug, info};

use crate::channels::{Channel, ChannelType};
use crate::tuner::windows::IBonDriver::{BonDriver, IBon};
use crate::tuner::{Tunable, Voltage};

mod IBonDriver;

struct BonDriverInner {
    dll_imported: ManuallyDrop<BonDriver>,
    pub interface: ManuallyDrop<IBon>,
}

impl Drop for BonDriverInner {
    fn drop(&mut self) {
        unsafe {
            //NOTE: The drop order should be explicitly defined like below
            ManuallyDrop::drop(&mut self.interface);
            ManuallyDrop::drop(&mut self.dll_imported);
        }
    }
}

impl AsyncRead for BonDriverInner {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<io::Result<usize>> {
        match self.interface.GetTsStream(buf) {
            Ok((recv, _)) if !recv.is_empty() => {
                debug!("{} bytes received.", recv.len());
                Poll::Ready(Ok(recv.len()))
            }
            Ok((recv, remaining)) if recv.is_empty() && remaining > 0 => {
                info!("{} remaining.", remaining);
                cx.waker().wake_by_ref();
                Poll::Pending
            }
            _ => {
                let w = cx.waker().clone();

                std::thread::spawn(move || {
                    // self.interface.WaitTsStream(Duration::from_millis(10));
                    std::thread::sleep(Duration::from_millis(100));
                    w.wake();
                });
                Poll::Pending
            }
        }
    }
}

pub struct UnTunedTuner {
    inner: BufReader<BonDriverInner>,
}

impl UnTunedTuner {
    pub fn new(path: String) -> Result<Self, io::Error> {
        let path_canonical = std::fs::canonicalize(path)?;

        let dll_imported = unsafe {
            info!("[BonDriver] Loading {:?}...", path_canonical);
            match BonDriver::new(path_canonical) {
                Ok(lib) => ManuallyDrop::new(lib),
                Err(e) => return Err(io::Error::new(io::ErrorKind::Unsupported, e)),
            }
        };

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

        Ok(Self {
            inner: BufReader::with_capacity(
                512 * 1024,
                BonDriverInner {
                    dll_imported,
                    interface,
                },
            ),
        })
    }

    pub fn enum_channels(&self, space: u32) -> Option<Vec<String>> {
        let interface = &self.inner.get_ref().interface;
        interface.EnumTuningSpace(space).and_then(|chs| {
            let mut ret = vec![chs];

            for i in 0..31 {
                if let Some(ch) = interface.EnumChannelName(space, i) {
                    ret.push(ch)
                } else if i == 0 {
                    return None;
                } else {
                    break;
                }
            }
            Some(ret)
        })
    }
}

impl Tunable for UnTunedTuner {
    fn tune(self, ch: Channel, lnb: Option<Voltage>) -> Result<Tuner, io::Error> {
        // Tune
        match &ch.ch_type {
            ChannelType::BonCh(phy_ch) => self.inner.get_ref().interface.SetChannel(*phy_ch)?,
            ChannelType::BonChSpace(space) => self
                .inner
                .get_ref()
                .interface
                .SetChannelBySpace(space.space, space.ch)?,
            other => {
                return Err(io::Error::new(
                    ErrorKind::Other,
                    format!("{:?} is not supported in Windows.", other),
                ))
            }
        }

        // LNB
        if lnb.is_some() {
            self.inner.get_ref().interface.SetLnbPower(1).unwrap();
        }

        Ok(Tuner {
            inner: self.inner,
            ch,
        })
    }
}

pub struct Tuner {
    inner: BufReader<BonDriverInner>,
    #[allow(unused)]
    ch: Channel,
}

impl Tunable for Tuner {
    fn tune(self, ch: Channel, lnb: Option<Voltage>) -> Result<Tuner, io::Error> {
        // Tune
        match &ch.ch_type {
            ChannelType::BonCh(phy_ch) => self.inner.get_ref().interface.SetChannel(*phy_ch)?,
            ChannelType::BonChSpace(space) => self
                .inner
                .get_ref()
                .interface
                .SetChannelBySpace(space.space, space.ch)?,
            _ => {}
        }

        // LNB
        if lnb.is_some() {
            self.inner.get_ref().interface.SetLnbPower(1).unwrap();
        }

        Ok(Tuner {
            inner: self.inner,
            ch,
        })
    }
}

impl Tuner {
    pub fn signal_quality(&self) -> f64 {
        self.inner
            .get_ref()
            .interface
            .GetSignalLevel()
            .unwrap()
            .into()
    }
}

impl AsyncRead for Tuner {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<io::Result<usize>> {
        Pin::new(&mut self.get_mut().inner).poll_read(cx, buf)
    }
}

impl AsyncBufRead for Tuner {
    fn poll_fill_buf(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<&[u8]>> {
        Pin::new(&mut self.get_mut().inner).poll_fill_buf(cx)
    }

    fn consume(self: Pin<&mut Self>, amt: usize) {
        Pin::new(&mut self.get_mut().inner).consume(amt)
    }
}
