#![allow(non_upper_case_globals, non_camel_case_types, non_snake_case)]

use std::ptr::NonNull;

mod error;

#[allow(clippy::all)]
include!(concat!(env!("OUT_DIR"), "/arib25_binding.rs"));

impl decoder {
    pub(super) fn new(ecm: bool, mut emm_recv_ids: Vec<i64>) -> Option<NonNull<decoder>> {
        let option = decoder_options {
            round: 4,
            strip: 0,
            emm: 0,
        };
        //TODO: emm_recv_ids は48ビットで切断
        let ptr = unsafe {
            if ecm || emm_recv_ids.len() > 1 {
                b25_startup_with_debug(
                    option,
                    if ecm { 1 } else { 0 },
                    B_CAS_ID {
                        count: emm_recv_ids.len() as i32,
                        data: emm_recv_ids.as_mut_ptr(),
                    },
                )
            } else {
                b25_startup(option)
            }
        };

        NonNull::new(ptr)
    }
    pub(super) fn push(&mut self, data: &[u8]) -> Option<ARIB_STD_B25_BUFFER> {
        let input = ARIB_STD_B25_BUFFER {
            data: data.as_ptr() as *mut _,
            size: data.len() as i32,
        };
        let out = unsafe { process_data(self as *mut decoder, input) };

        if out.size > 0 {
            Some(out)
        } else {
            None
        }
    }
    pub(super) fn clean_up(&mut self) {
        unsafe { b25_shutdown(self as *mut decoder) };
    }
}
