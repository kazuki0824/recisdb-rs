use std::io::{BufRead, Read, Write};
use crate::bindings::arib_std_b25::ARIB_STD_B25_BUFFER;

mod arib_std_b25;

struct InnerDecoder {
    dec: arib_std_b25::ARIB_STD_B25,
    cas: arib_std_b25::B_CAS_CARD
}
impl InnerDecoder {
    fn new(cas: arib_std_b25::B_CAS_CARD) -> Result<Self, ()> {
        Ok(Self {
            dec: arib_std_b25::ARIB_STD_B25::new(cas)?,
            cas,
        })
    }
    fn clean_up(&mut self) {
        self.dec.clean_up();
    }
}

impl Write for InnerDecoder {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let buffer_struct = ARIB_STD_B25_BUFFER {
            data: buf.as_mut_ptr(),
            size: 0
        };
        unsafe {
            let put = self.dec.put.unwrap();
            put()
        }

    }

    fn flush(&mut self) -> std::io::Result<()> {
        todo!()
    }
}

impl Read for InnerDecoder {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        todo!()
    }
}
