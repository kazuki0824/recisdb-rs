use crate::channels::{Channel, ChannelType};
use crate::tuner::Voltage;
use dvbv5::{DmxFd, FilePtr, FrontendId, FrontendParametersPtr};
use dvbv5_sys::fe_status::FE_HAS_LOCK;
use dvbv5_sys::{
    dmx_output, dmx_ts_pes, fe_delivery_system, fe_status, DTV_BANDWIDTH_HZ, DTV_FREQUENCY,
    DTV_STATUS,
};
use futures_util::io::{AllowStdIo, BufReader};
use futures_util::{AsyncBufRead, AsyncRead};
use std::ffi::c_uint;
use std::fs::File;
use std::io::{Error, Write};
use std::path::Path;
use std::pin::Pin;
use std::task::{Context, Poll};

pub struct UnTunedTuner {
    id: (u8, u8),
    frontend: FrontendParametersPtr,
    demux: DmxFd,
    isdb_s: FilePtr,
    isdb_t: FilePtr,
}

impl UnTunedTuner {
    pub fn new(adapter_number: u8, fe_number: u8) -> Result<Self, Error> {
        let (frontend, demux) = {
            let frontend_id = FrontendId {
                adapter_number,
                frontend_number: fe_number,
            };

            let f = FrontendParametersPtr::new(&frontend_id, Some(3), Some(false))
                .expect("Something went wrong while opening DVB frontend.");
            let d = DmxFd::new(&frontend_id).expect("Failed to open the demuxer");

            (f, d)
        };

        // Ch tables
        let isdb_s = {
            let settings = include_str!("./dvbv5/dvbv5_channels_isdbs.conf");
            let tmp_path = "/tmp/dvbv5_channels_isdbs.conf";

            if !Path::exists(tmp_path.as_ref()) {
                let mut f = File::create(tmp_path)?;
                write!(f, "{settings}").expect(&format!("Write to {tmp_path} failed."));
            }

            FilePtr::new(tmp_path.as_ref(), None, None).unwrap()
        };

        let isdb_t = {
            let settings = include_str!("./dvbv5/dvbv5_channels_isdbt.conf");
            let tmp_path = "/tmp/dvbv5_channels_isdbt.conf";

            if !Path::exists(tmp_path.as_ref()) {
                let mut f = File::create(tmp_path)?;
                write!(f, "{settings}").expect(&format!("Write to {tmp_path} failed."));
            }

            FilePtr::new(tmp_path.as_ref(), None, None).unwrap()
        };

        Ok(Self {
            id: (adapter_number, fe_number),
            frontend,
            demux,
            isdb_s,
            isdb_t,
        })
    }

    pub fn tune(self, ch: Channel, lnb: Option<Voltage>) -> Result<Tuner, Error> {
        // Spec verification
        let ch_checked = {
            let sys = self.frontend.get_current_sys();

            match (ch.ch_type, sys) {
                (ChannelType::Terrestrial(_), fe_delivery_system::SYS_ISDBT)
                | (ChannelType::Catv(_), fe_delivery_system::SYS_ISDBT) => Some(491142857),
                (ChannelType::BS(_, _), fe_delivery_system::SYS_ISDBS)
                | (ChannelType::CS(_), fe_delivery_system::SYS_ISDBS) => Some(0),
                _ => None,
            }
        }
        .unwrap();

        // DELIVERY_SYSTEM
        // FREQUENCY
        // BANDWIDTH_HZ
        let result = unsafe {
            let p = self.frontend.get_c_ptr();
            dvbv5_sys::dvb_fe_store_parm(p, DTV_FREQUENCY as c_uint, ch_checked);
            if let Some(bw) = Some(6000000) {
                dvbv5_sys::dvb_fe_store_parm(p, DTV_BANDWIDTH_HZ as c_uint, bw);
            }
            let mut stat: fe_status = fe_status::FE_NONE;
            dvbv5_sys::dvb_fe_retrieve_stats(
                p,
                DTV_STATUS as c_uint,
                &mut stat as *mut fe_status as *mut _,
            );
            (stat as u8 & FE_HAS_LOCK as u8) != 0
        };

        let result = unsafe {
            dvbv5_sys::dvb_set_pesfilter(
                self.demux.as_raw_fd(),
                0x2000,
                dmx_ts_pes::DMX_PES_VIDEO0,
                dmx_output::DMX_OUT_TS_TAP,
                100000,
            ) != 0
        };

        let f = File::open(format!("/dev/dvb/adapter{}/dvr{}", self.id.0, self.id.1))?;
        Ok(Tuner {
            inner: self,
            stream: BufReader::new(AllowStdIo::new(f)),
        })
    }
}

pub struct Tuner {
    inner: UnTunedTuner,
    stream: BufReader<AllowStdIo<File>>,
}

impl AsyncRead for Tuner {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<std::io::Result<usize>> {
        Pin::new(&mut self.get_mut().stream).poll_read(cx, buf)
    }
}

impl AsyncBufRead for Tuner {
    fn poll_fill_buf(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<&[u8]>> {
        Pin::new(&mut self.get_mut().stream).poll_fill_buf(cx)
    }

    fn consume(self: Pin<&mut Self>, amt: usize) {
        Pin::new(&mut self.get_mut().stream).consume(amt)
    }
}
