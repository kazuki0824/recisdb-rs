use std::fs::File;
use std::os::fd::{AsRawFd, RawFd};
use std::pin::Pin;
use std::task::{Context, Poll};

use futures_util::io::{AllowStdIo, BufReader};
use futures_util::{AsyncBufRead, AsyncRead};
use log::warn;

use crate::channels::output::IoctlFreq;
use crate::channels::{Channel, ChannelType};
use crate::tuner::Voltage;

use super::threaded_reader::ThreadedReader;

nix::ioctl_write_ptr!(set_ch, 0x8d, 0x01, IoctlFreq);
nix::ioctl_none!(start_rec, 0x8d, 0x02);
nix::ioctl_none!(stop_rec, 0x8d, 0x03);
nix::ioctl_read!(ptx_get_cnr, 0x8d, 0x04, i64);
nix::ioctl_write_int!(ptx_enable_lnb, 0x8d, 0x05);
nix::ioctl_none!(ptx_disable_lnb, 0x8d, 0x06);
nix::ioctl_write_int!(ptx_set_sys_mode, 0x8d, 0x0b);

pub struct UnTunedTuner {
    file: File,
    buf_sz: usize,
}

impl UnTunedTuner {
    pub fn new(path: String, buf_sz: usize) -> Result<Self, std::io::Error> {
        let path = std::fs::canonicalize(path)?;
        let file = std::fs::OpenOptions::new().read(true).open(path)?;
        Ok(Self { file, buf_sz })
    }
    pub fn tune(self, ch: Channel, lnb: Option<Voltage>) -> Result<Tuner, std::io::Error> {
        // Clone the file descriptor: one copy is kept for ioctl operations
        // (tuning, signal quality, LNB control), and the original is moved
        // into the ThreadedReader for continuous data streaming.
        // Using try_clone() (dup) is safe for Linux device files and allows
        // concurrent read + ioctl from different threads.
        let ioctl_file = self.file.try_clone()?;

        let _errno = unsafe { set_ch(ioctl_file.as_raw_fd(), &ch.ch_type.clone().into())? };

        let _errno = match lnb {
            Some(Voltage::_11v) => unsafe { ptx_enable_lnb(ioctl_file.as_raw_fd(), 1)? },
            Some(Voltage::_15v) => unsafe { ptx_enable_lnb(ioctl_file.as_raw_fd(), 2)? },
            _ => unsafe { ptx_disable_lnb(ioctl_file.as_raw_fd())? },
        };

        let _errno = unsafe { start_rec(ioctl_file.as_raw_fd())? };

        let lnb_capab = match lnb {
            None | Some(Voltage::Low) => None,
            _ => Some(PowerOffHandle {
                fd: ioctl_file.as_raw_fd(),
                is_disarmed: false,
            }),
        };

        // Wrap the data file in ThreadedReader so that a dedicated background
        // thread continuously drains the kernel's tuner buffer, preventing
        // data drops when the downstream decoder pipeline blocks (e.g.,
        // during B-CAS card ECM processing).
        let reader = ThreadedReader::with_defaults(self.file)?;

        Ok(Tuner {
            // Field order matters for drop safety: _lnb_capab is dropped
            // first (calls ptx_disable_lnb via ioctl_file's fd), then
            // ioctl_file is dropped (closes the fd). inner (ThreadedReader)
            // is dropped last, which terminates the reader thread.
            _lnb_capab: lnb_capab,
            ioctl_file,
            inner: BufReader::with_capacity(self.buf_sz, AllowStdIo::new(reader)),
            channel: ch,
        })
    }
}

pub(crate) struct PowerOffHandle {
    fd: RawFd,
    is_disarmed: bool,
}

pub struct Tuner {
    _lnb_capab: Option<PowerOffHandle>,
    // Duplicated file descriptor for ioctl operations (signal quality,
    // re-tuning, LNB control). This fd points to the same underlying
    // device as the one moved into the ThreadedReader, but is independent
    // and can be used from the main thread concurrently with the reader
    // thread's read operations.
    ioctl_file: File,
    inner: BufReader<AllowStdIo<ThreadedReader>>,
    channel: Channel,
}

impl Tuner {
    pub fn signal_quality(&self) -> f64 {
        let raw = {
            let mut raw = [0i64; 1];
            let ioctl_result =
                unsafe { ptx_get_cnr(self.ioctl_file.as_raw_fd(), &mut raw[0]) };
            if let Err(error) = ioctl_result {
                warn!("Failed to get CNR from tuner device: {error}");
                return 0.0;
            }
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
    fn tune(mut self, ch: Channel, lnb: Option<Voltage>) -> Result<Tuner, std::io::Error> {
        let _errno =
            unsafe { set_ch(self.ioctl_file.as_raw_fd(), &ch.ch_type.clone().into())? };

        let _errno = match lnb {
            Some(Voltage::_11v) => unsafe { ptx_enable_lnb(self.ioctl_file.as_raw_fd(), 1)? },
            Some(Voltage::_15v) => unsafe { ptx_enable_lnb(self.ioctl_file.as_raw_fd(), 2)? },
            _ => unsafe { ptx_disable_lnb(self.ioctl_file.as_raw_fd())? },
        };

        if let Some(old_lnb_capab) = self._lnb_capab.as_mut() {
            old_lnb_capab.is_disarmed = true;
        }

        self._lnb_capab = match lnb {
            None | Some(Voltage::Low) => None,
            _ => Some(PowerOffHandle {
                fd: self.ioctl_file.as_raw_fd(),
                is_disarmed: false,
            }),
        };
        self.channel = ch;

        Ok(self)
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
        if self.is_disarmed {
            return;
        }

        let disable_result = unsafe { ptx_disable_lnb(self.fd) };
        if let Err(error) = disable_result {
            warn!("Failed to disable LNB in PowerOffHandle::drop: {error}");
        }
    }
}
