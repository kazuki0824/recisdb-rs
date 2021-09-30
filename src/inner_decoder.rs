#![allow(non_upper_case_globals, non_camel_case_types, non_snake_case)]

mod error;

use std::ptr::NonNull;

include!(concat!(env!("OUT_DIR"), "/arib25_binding.rs"));

impl decoder {
    pub(super) fn new(emm: bool) -> Option<NonNull<decoder>> {
        let option = decoder_options {
            round: 4,
            strip: 0,
            emm: 0,
        };
        unsafe {
            let ptr = b25_startup(option, emm as i32);
            NonNull::new(ptr)
        }
    }
    pub(super) fn push(&mut self, data: &mut [u8]) -> Option<ARIB_STD_B25_BUFFER> {
        let input = ARIB_STD_B25_BUFFER {
            data: data.as_mut_ptr(),
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
