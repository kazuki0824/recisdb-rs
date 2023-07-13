use std::io;
use std::mem::ManuallyDrop;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::Duration;

use futures_util::io::BufReader;
use futures_util::{AsyncBufRead, AsyncRead};
use log::info;

use crate::channels::{Channel, ChannelType};
use crate::tuner::windows::IBonDriver::{BonDriver, IBon};
use crate::tuner::{Tunable, Voltage};

mod IBonDriver;

struct BonDriverInner {
    dll_imported: ManuallyDrop<BonDriver>,
    pub interface: ManuallyDrop<IBon<10000>>,
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
        match self.interface.GetTsStream() {
            Ok((recv, _)) if !recv.is_empty() => {
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
            inner: BufReader::new(BonDriverInner {
                dll_imported,
                interface,
            }),
        })
    }
}

impl Tunable for UnTunedTuner {
    fn tune(self, ch: Channel, lnb: Option<Voltage>) -> Result<Tuner, io::Error> {
        // Tune
        if let Some(phy_ch) = ch.clone().try_get_physical_num() {
            self.inner.get_ref().interface.SetChannel(phy_ch)?;
        } else if let ChannelType::Bon(space) = ch.clone().ch_type {
            self.inner
                .get_ref()
                .interface
                .SetChannelBySpace(space.space, space.ch)?;
        }

        // LNB
        if matches!((&ch.ch_type, lnb), (ChannelType::BS(..) | ChannelType::CS(_), Some(_))) {
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
    ch: Channel,
}

impl Tunable for Tuner {
    fn tune(self, ch: Channel, lnb: Option<Voltage>) -> Result<Tuner, io::Error> {
        // Tune
        if let Some(phy_ch) = ch.clone().try_get_physical_num() {
            self.inner.get_ref().interface.SetChannel(phy_ch)?;
        } else if let ChannelType::Bon(space) = ch.clone().ch_type {
            self.inner
                .get_ref()
                .interface
                .SetChannelBySpace(space.space, space.ch)?;
        }

        // LNB
        if matches!((&ch.ch_type, lnb), (ChannelType::BS(..) | ChannelType::CS(_), Some(_))) {
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
        todo!()
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
