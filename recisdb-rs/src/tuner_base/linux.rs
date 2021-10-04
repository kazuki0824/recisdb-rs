use nix::{fcntl, sys};
use std::error::Error;
use std::os::unix::io::FromRawFd;
use futures::AsyncRead;
use futures::io::AllowStdIo;
use crate::channels::{Channel, ChannelType, Freq};
nix::ioctl_write_ptr!(set_ch, 0x8d, 0x01, Freq);
nix::ioctl_none!(lnb_dis, 0x8d, 0x03);
nix::ioctl_read!(ptx_get_cnr, 0x8d, 0x04, u8);
nix::ioctl_write_int!(ptx_enable_lnb, 0x8d, 0x05);
nix::ioctl_none!(ptx_disable_lnb, 0x8d, 0x06);
nix::ioctl_write_int!(ptx_set_sys_mode, 0x8d, 0x0b);

pub struct TunedDevice {
    handle: std::os::unix::io::RawFd,
    channel: Channel
}
impl TunedDevice
{
    pub fn tune(path: &str, channel: Channel, offset_k_hz: i32) -> Result<Self, Box<dyn Error>>
    {
        let handle = fcntl::open(path, fcntl::OFlag::O_RDONLY, sys::stat::Mode::empty())?;
        unsafe { set_ch(handle, &channel.to_freq(offset_k_hz))? };
        Ok(Self {
            handle,
            channel
        })
    }
}

impl super::Tuned for TunedDevice {
    fn signal_quality(&self) -> f64 {
        let raw: u8 = 0;
        let errno = unsafe { ptx_get_cnr(self.handle, raw as *mut u8) }.unwrap();

        match self.channel.ch_type  {
            ChannelType::Terrestrial =>{
                let p = (5505024.0 / (raw as f64)).log10() * 10.0;
                let cnr = (0.000024 * p * p * p * p) - (0.0016 * p * p * p) +
                    (0.0398 * p * p) + (0.5491 * p)+3.0965;
                cnr
            }
            _ => {
                todo!("ISDB-S sn rate");
                0.0
            }
        }
    }

    fn set_lnb(&self) -> Result<i8, String> {
        todo!()
    }

    fn open(self) -> Box<dyn AsyncRead + Unpin> {
        let raw = unsafe { std::fs::File::from_raw_fd(self.handle) };
        Box::new(AllowStdIo::new(raw))
    }
}
