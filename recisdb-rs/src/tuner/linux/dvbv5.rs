use crate::channels::output::DvbFreq;
use crate::channels::{Channel, ChannelType};
use crate::tuner::Voltage;
use dvbv5::{DmxFd, FrontendId, FrontendParametersPtr};
use dvbv5_sys::fe_delivery_system::{SYS_ISDBS, SYS_ISDBT};
use dvbv5_sys::fe_sec_voltage::{SEC_VOLTAGE_13, SEC_VOLTAGE_18, SEC_VOLTAGE_OFF};
use dvbv5_sys::fe_status::{self, FE_HAS_LOCK};
use dvbv5_sys::{
    dmx_output, dmx_ts_pes, dvb_set_compat_delivery_system, DTV_BANDWIDTH_HZ, DTV_FREQUENCY,
    DTV_ISDBT_LAYER_ENABLED, DTV_ISDBT_PARTIAL_RECEPTION, DTV_ISDBT_SOUND_BROADCASTING, DTV_STATUS,
    DTV_STREAM_ID, DTV_VOLTAGE, NO_STREAM_ID_FILTER,
};
use futures_util::io::{AllowStdIo, BufReader};
use futures_util::{AsyncBufRead, AsyncRead};
use log::info;
use std::ffi::c_uint;
use std::fs::File;
use std::io::Error;
use std::pin::Pin;
use std::task::{Context, Poll};

pub struct UnTunedTuner {
    id: (u8, u8),
    frontend: FrontendParametersPtr,
    demux: DmxFd,
}

impl UnTunedTuner {
    pub fn new(adapter_number: u8, fe_number: u8) -> Result<Self, Error> {
        let (frontend, demux) = {
            let frontend_id = FrontendId {
                adapter_number,
                frontend_number: fe_number,
            };

            let f = FrontendParametersPtr::new(&frontend_id, Some(1), Some(false))
                .expect("Something went wrong while opening DVB frontend.");
            let d = DmxFd::new(&frontend_id).expect("Failed to open the demuxer");

            (f, d)
        };

        Ok(Self {
            id: (adapter_number, fe_number),
            frontend,
            demux,
        })
    }

    pub fn tune(self, ch: Channel, lnb: Option<Voltage>) -> Result<Tuner, Error> {
        const WAIT_DUR: std::time::Duration = std::time::Duration::from_secs(1);

        // fe
        let _result = unsafe {
            let sys = self.frontend.get_current_sys();
            let p = self.frontend.get_c_ptr();

            let raw_freq: DvbFreq = ch.ch_type.clone().into();

            dvb_set_compat_delivery_system(p, sys as u32);
            match (ch.ch_type, sys) {
                (ChannelType::Terrestrial(..), SYS_ISDBT) | (ChannelType::Catv(..), SYS_ISDBT) => {
                    dvbv5_sys::dvb_fe_store_parm(p, DTV_FREQUENCY as c_uint, raw_freq.freq_hz);
                    dvbv5_sys::dvb_fe_store_parm(p, DTV_BANDWIDTH_HZ as c_uint, 6000000);

                    dvbv5_sys::dvb_fe_store_parm(p, DTV_ISDBT_PARTIAL_RECEPTION, 0);
                    dvbv5_sys::dvb_fe_store_parm(p, DTV_ISDBT_SOUND_BROADCASTING, 0);
                    dvbv5_sys::dvb_fe_store_parm(p, DTV_ISDBT_LAYER_ENABLED, 0x07);

                    dvbv5_sys::dvb_fe_set_parms(p)
                }
                (ChannelType::BS(..), SYS_ISDBS) | (ChannelType::CS(..), SYS_ISDBS) => {
                    dvbv5_sys::dvb_fe_store_parm(p, DTV_FREQUENCY as c_uint, raw_freq.freq_hz);
                    dvbv5_sys::dvb_fe_store_parm(
                        p,
                        DTV_STREAM_ID as c_uint,
                        raw_freq.stream_id.unwrap(),
                    );
                    match lnb {
                        Some(Voltage::High11v) => {
                            dvbv5_sys::dvb_fe_store_parm(p, DTV_VOLTAGE, SEC_VOLTAGE_13 as u32)
                        }
                        Some(Voltage::High15v) => {
                            dvbv5_sys::dvb_fe_store_parm(p, DTV_VOLTAGE, SEC_VOLTAGE_18 as u32)
                        }
                        _ => 0,
                    };

                    dvbv5_sys::dvb_fe_set_parms(p)
                }
                _ => panic!("Wrong frontend specified"),
            };

            let mut stat: fe_status = fe_status::FE_NONE;
            let mut _res = 0;
            while (stat as u8 & FE_HAS_LOCK as u8) == 0 {
                std::thread::sleep(WAIT_DUR);
                _res = dvbv5_sys::dvb_fe_get_stats(p);
                _res = dvbv5_sys::dvb_fe_retrieve_stats(
                    p,
                    DTV_STATUS as c_uint,
                    &mut stat as *mut fe_status as *mut _,
                );
                info!("Check signal level")
            }
        };
        // dmx
        unsafe {
            dvbv5_sys::dvb_set_pesfilter(
                self.demux.as_raw_fd(),
                0x2000,
                dmx_ts_pes::DMX_PES_OTHER,
                dmx_output::DMX_OUT_TS_TAP,
                8192,
            );
            // dvbv5_sys::dvb_set_section_filter(
            //     self.demux.as_raw_fd(),
            //     0x2000,
            //     18,
            //     null_mut() as *mut _,
            //     null_mut() as *mut _,
            //     null_mut() as *mut _,
            //     DMX_IMMEDIATE_START | DMX_CHECK_CRC
            // );
        }

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
