use std::fs::File;
use std::os::fd::{AsRawFd, RawFd};
use std::pin::Pin;
use std::task::{Context, Poll};

use futures_util::io::{AllowStdIo, BufReader};
use futures_util::{AsyncBufRead, AsyncRead};

use crate::channels::output::IoctlFreq;
use crate::channels::{Channel, ChannelType};
use crate::tuner::Voltage;

nix::ioctl_write_ptr!(set_ch, 0x8d, 0x01, IoctlFreq);
nix::ioctl_none!(start_rec, 0x8d, 0x02);
nix::ioctl_none!(stop_rec, 0x8d, 0x03);
nix::ioctl_read!(ptx_get_cnr, 0x8d, 0x04, i64);
nix::ioctl_write_int!(ptx_enable_lnb, 0x8d, 0x05);
nix::ioctl_none!(ptx_disable_lnb, 0x8d, 0x06);
nix::ioctl_write_int!(ptx_set_sys_mode, 0x8d, 0x0b);

pub struct UnTunedTuner {
    inner: BufReader<AllowStdIo<File>>,
}

impl UnTunedTuner {
    pub fn new(path: String) -> Result<Self, std::io::Error> {
        let path = std::fs::canonicalize(path)?;
        let f = std::fs::OpenOptions::new().read(true).open(path)?;
        let buf_sz = std::env::var("RECISDB_INPUT_BUF").and_then(|s| s.parse::<usize>()).ok();

        Ok(Self {
            inner: BufReader::with_capacity(buf_sz.unwrap_or(200000), AllowStdIo::new(f)),
        })
    }
    pub fn tune(self, ch: Channel, lnb: Option<Voltage>) -> Result<Tuner, std::io::Error> {
        let f = self.inner.get_ref().get_ref();

        let _errno = unsafe { set_ch(f.as_raw_fd(), &ch.ch_type.clone().into())? };

        let _errno = match lnb {
            Some(Voltage::_11v) => unsafe { ptx_enable_lnb(f.as_raw_fd(), 1)? },
            Some(Voltage::_15v) => unsafe { ptx_enable_lnb(f.as_raw_fd(), 2)? },
            _ => unsafe { ptx_disable_lnb(f.as_raw_fd())? },
        };

        let _errno = unsafe { start_rec(f.as_raw_fd()) }.unwrap();

        let lnb_capab = match lnb {
            None | Some(Voltage::Low) => None,
            _ => Some(PowerOffHandle { fd: f.as_raw_fd() }),
        };

        Ok(Tuner {
            inner: self.inner,
            channel: ch,
            _lnb_capab: lnb_capab,
        })
    }
}

pub(crate) struct PowerOffHandle {
    fd: RawFd,
}

pub struct Tuner {
    _lnb_capab: Option<PowerOffHandle>,
    inner: BufReader<AllowStdIo<File>>,
    channel: Channel,
}

impl Tuner {
    pub fn signal_quality(&self) -> f64 {
        let raw = {
            let mut raw = [0i64; 1];
            let f = self.inner.get_ref().get_ref();

            let _errno = unsafe { ptx_get_cnr(f.as_raw_fd(), &mut raw[0]) }.unwrap();
            raw[0]
        };

        match self.channel.ch_type {
            ChannelType::Terrestrial(..) => {
                let p = (5505024.0 / (raw as f64)).log10() * 10.0;
                (0.000024 * p * p * p * p) - (0.0016 * p * p * p)
                    + (0.0398 * p * p)
                    + (0.5491 * p)
                    + 3.0965
            }
            _ => {
                const AF_LEVEL_TABLE: [f64; 14] = [
                    24.07, // 00    00    0        24.07dB
                    24.07, // 10    00    4096     24.07dB
                    18.61, // 20    00    8192     18.61dB
                    15.21, // 30    00    12288    15.21dB
                    12.50, // 40    00    16384    12.50dB
                    10.19, // 50    00    20480    10.19dB
                    8.140, // 60    00    24576    8.140dB
                    6.270, // 70    00    28672    6.270dB
                    4.550, // 80    00    32768    4.550dB
                    3.730, // 88    00    34816    3.730dB
                    3.630, // 88    FF    35071    3.630dB
                    2.940, // 90    00    36864    2.940dB
                    1.420, // A0    00    40960    1.420dB
                    0.000, // B0    00    45056    -0.01dB
                ];
                let sig = ((raw & 0xFF00) >> 8) as u8;
                if sig <= 0x10u8 {
                    /* clipped maximum */
                    24.07
                } else if sig >= 0xB0u8 {
                    /* clipped minimum */
                    0.0
                } else {
                    /* linear interpolation */
                    let f_mix_rate = (((sig as u16 & 0x0F) << 8) | sig as u16) as f64 / 4096.0;
                    AF_LEVEL_TABLE[(sig >> 4) as usize] * (1.0 - f_mix_rate)
                        + AF_LEVEL_TABLE[(sig >> 4) as usize + 0x01] * f_mix_rate
                }
            }
        }
    }
    fn tune(self, ch: Channel, lnb: Option<Voltage>) -> Result<Tuner, std::io::Error> {
        let f = self.inner.get_ref().get_ref();

        let _errno = unsafe { set_ch(f.as_raw_fd(), &ch.ch_type.clone().into())? };

        let _errno = match lnb {
            Some(Voltage::_11v) => unsafe { ptx_enable_lnb(f.as_raw_fd(), 1)? },
            Some(Voltage::_15v) => unsafe { ptx_enable_lnb(f.as_raw_fd(), 2)? },
            _ => unsafe { ptx_disable_lnb(f.as_raw_fd())? },
        };

        let lnb_capab = match lnb {
            None | Some(Voltage::Low) => None,
            _ => Some(PowerOffHandle { fd: f.as_raw_fd() }),
        };

        Ok(Tuner {
            inner: self.inner,
            channel: ch,
            _lnb_capab: lnb_capab,
        })
    }
}

impl AsyncRead for Tuner {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<std::io::Result<usize>> {
        Pin::new(&mut self.get_mut().inner).poll_read(cx, buf)
    }
}

impl AsyncBufRead for Tuner {
    fn poll_fill_buf(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<&[u8]>> {
        Pin::new(&mut self.get_mut().inner).poll_fill_buf(cx)
    }

    fn consume(self: Pin<&mut Self>, amt: usize) {
        Pin::new(&mut self.get_mut().inner).consume(amt)
    }
}

impl Drop for PowerOffHandle {
    fn drop(&mut self) {
        unsafe {
            ptx_disable_lnb(self.fd.clone()).unwrap();
        }
    }
}
