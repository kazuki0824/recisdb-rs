use std::error::Error;
use std::os::unix::io::AsRawFd;

use futures::AsyncBufRead;

use crate::channels::{Channel, ChannelType, Freq};

nix::ioctl_write_ptr!(set_ch, 0x8d, 0x01, Freq);
nix::ioctl_none!(lnb_dis, 0x8d, 0x03);
nix::ioctl_read!(ptx_get_cnr, 0x8d, 0x04, u8);
nix::ioctl_write_int!(ptx_enable_lnb, 0x8d, 0x05);
nix::ioctl_none!(ptx_disable_lnb, 0x8d, 0x06);
nix::ioctl_write_int!(ptx_set_sys_mode, 0x8d, 0x0b);

pub struct TunedDevice {
    pub f: std::fs::File,
    channel: Channel,
}
impl TunedDevice {
    pub fn tune(path: &str, channel: Channel, offset_k_hz: i32) -> Result<Self, Box<dyn Error>> {
        let mut f = std::fs::OpenOptions::new().read(true).open(path).unwrap();
        unsafe { set_ch(f.as_raw_fd(), &channel.to_freq(offset_k_hz))? };
        //Warm-up
        let mut e = [0u8; 2];
        use std::io::Read;
        {
            let mut result = f.read_exact(&mut e[0..]);
            let mut i = 0;
            while result.is_err() && (0..20).contains(&i)
            {
                i += 1;
                result = f.read_exact(&mut e[0..]);
            }
            result
        }.expect("The file was definitely opened and the channel selection was successful,\nbut the stream cannot be read properly.\n");

        Ok(Self { f, channel })
    }
}

impl super::Tuned for TunedDevice {
    fn signal_quality(&self) -> f64 {
        let raw: u8 = 0;
        let errno = unsafe { ptx_get_cnr(self.f.as_raw_fd(), raw as *mut u8) }.unwrap();

        match self.channel.ch_type {
            ChannelType::Terrestrial => {
                let p = (5505024.0 / (raw as f64)).log10() * 10.0;
                let cnr = (0.000024 * p * p * p * p) - (0.0016 * p * p * p)
                    + (0.0398 * p * p)
                    + (0.5491 * p)
                    + 3.0965;
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

    fn open_stream(self) -> Box<dyn AsyncBufRead + Unpin> {
        use futures::io::AllowStdIo;
        use std::io::BufReader;

        let with_buffer = BufReader::new(self.f);
        Box::new(AllowStdIo::new(with_buffer))
    }
}
