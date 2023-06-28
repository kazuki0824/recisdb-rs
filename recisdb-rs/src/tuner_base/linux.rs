use b25_sys::futures_io::AsyncBufRead;
use futures_util::io::AllowStdIo;
use std::error::Error;
use std::os::unix::io::AsRawFd;

use crate::channels::{Channel, ChannelType, Freq};
use crate::tuner_base::Voltage;

nix::ioctl_write_ptr!(set_ch, 0x8d, 0x01, Freq);
nix::ioctl_none!(start_rec, 0x8d, 0x02);
nix::ioctl_none!(stop_rec, 0x8d, 0x03);
nix::ioctl_read!(ptx_get_cnr, 0x8d, 0x04, i64);
nix::ioctl_write_int!(ptx_enable_lnb, 0x8d, 0x05);
nix::ioctl_none!(ptx_disable_lnb, 0x8d, 0x06);
nix::ioctl_write_int!(ptx_set_sys_mode, 0x8d, 0x0b);

pub struct TunedDevice {
    pub f: std::fs::File,
    channel: Channel,
}
impl TunedDevice {
    pub fn tune(
        path: &str,
        channel: Channel,
        offset_k_hz: i32,
        voltage: Option<Voltage>,
    ) -> Result<Self, Box<dyn Error>> {
        let path = std::fs::canonicalize(path)?;
        let f = std::fs::OpenOptions::new().read(true).open(path)?;
        let _errno = unsafe { set_ch(f.as_raw_fd(), &channel.to_ioctl_freq(offset_k_hz))? };

        match voltage {
            Some(Voltage::High11v) => {
                let errno = unsafe { ptx_enable_lnb(f.as_raw_fd(), 1) }.unwrap();
            }
            Some(Voltage::High15v) => {
                let errno = unsafe { ptx_enable_lnb(f.as_raw_fd(), 2) }.unwrap();
            }
            _ => {
                let errno = unsafe { ptx_disable_lnb(f.as_raw_fd()) }.unwrap();
            }
        }
        Ok(Self { f, channel })
    }
}

impl super::Tuned for TunedDevice {
    fn signal_quality(&self) -> f64 {
        let raw = {
            let mut raw = [0i64; 1];
            let _errno = unsafe { ptx_get_cnr(self.f.as_raw_fd(), &mut raw[0]) }.unwrap();
            raw[0]
        };

        match self.channel.ch_type {
            ChannelType::Terrestrial(_) => {
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

    fn open_stream(mut self) -> Box<dyn AsyncBufRead + Unpin> {
        use std::io::BufReader;

        unsafe { start_rec(self.f.as_raw_fd()) }.unwrap();
        //Warm-up
        let mut e = [0u8; 2];
        use std::io::Read;
        {
            let mut result = self.f.read_exact(&mut e[0..]);
            let mut i = 0;
            while result.is_err() && (0..20).contains(&i)
            {
                i += 1;
                result = self.f.read_exact(&mut e[0..]);
            }
            result
        }.expect("The file was definitely opened and the channel selection was successful,\nbut the stream cannot be read properly.\n");
        //Init buffered io
        let with_buffer = BufReader::new(self.f);
        Box::new(AllowStdIo::new(with_buffer))
    }
}
