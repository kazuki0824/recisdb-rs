use pin_project_lite::pin_project;
use std::io::{Read, Write};
use std::ptr::null_mut;
use std::ptr::NonNull;

use crate::bindings::arib_std_b25::{ARIB_STD_B25, ARIB_STD_B25_BUFFER, B_CAS_CARD};
use crate::bindings::error::AribB25DecoderError;

mod arib_std_b25;
mod error;
mod ffi;

pin_project! {
    pub(crate) struct InnerDecoder {
        #[pin]
        pub dec: NonNull<ARIB_STD_B25>,
        #[pin]
        cas: Option<Box<B_CAS_CARD>>,
    }
}
// impl PinnedDrop for InnerDecoder<'_> {
//     fn drop(self: Pin<&mut self>) {
//         //Release the decoder instance
//         self.cas.take().map(|cas| {
//             cas.get_ref()
//         }).map(|cas| {
//             unsafe { cas.release.unwrap()(cas as *const B_CAS_CARD as *mut ::std::os::raw::c_void) };
//         });
//     }
// }
impl InnerDecoder {
    pub(crate) unsafe fn new(key: bool) -> Result<Self, AribB25DecoderError> {
        let dec = arib_std_b25::create_arib_std_b25();

        // Clone the instance from the original that starts from the address created by create_arib_std_b25()
        // If the program crashes when this instance is freed, this code is the cause of the crash.
        match key {
            false => {
                let cas = arib_std_b25::create_b_cas_card();
                if cas.is_null() {
                    Err(AribB25DecoderError::ARIB_STD_B25_ERROR_EMPTY_B_CAS_CARD)
                } else {
                    // Initialize the CAS card
                    (*cas).initialize();
                    (*dec).set_b_cas_card(&*cas);
                    Ok(Self {
                        dec: NonNull::new(dec).unwrap(),
                        cas: None,
                    })
                }
            }
            true => {
                let mut cas = B_CAS_CARD::default();
                //Allocate private data inside B_CAS_CARD
                cas.initialize();
                let ret = Self {
                    dec: NonNull::new(dec).unwrap(),
                    cas: Some(Box::new(cas)),
                };
                ret.dec.as_ref().set_b_cas_card(ret.cas.as_ref().unwrap());
                Ok(ret)
            }
        }
    }
}

impl Write for InnerDecoder {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let code = unsafe {
            let buffer_struct = ARIB_STD_B25_BUFFER {
                data: std::mem::transmute::<*const u8, *mut u8>(buf.as_ptr()),
                size: buf.len() as u32,
            };
            self.dec.as_ref().put(&buffer_struct)
        };

        match code {
            0 => Ok(buf.len()),
            _ => {
                let err = AribB25DecoderError::from(code);
                // if greater than 0, it means that the decoder emitted some warnings.
                // if less than 0, it means that the decoder emitted some errors.
                if code > 0 {
                    eprintln!("{}", err);
                    Ok(buf.len())
                } else {
                    Err(std::io::Error::new(std::io::ErrorKind::Other, err))
                }
            }
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        let code = unsafe { self.dec.as_ref().flush() };

        match code {
            0 => Ok(()),
            _ => {
                let err = AribB25DecoderError::from(code);
                eprintln!("{}", err);
                // if greater than 0, it means that the decoder emitted some warnings.
                // if less than 0, it means that the decoder emitted some errors.
                if code > 0 {
                    Ok(())
                } else {
                    Err(std::io::Error::new(std::io::ErrorKind::Other, err))
                }
            }
        }
    }
}

impl Read for InnerDecoder {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let (code, sz) = unsafe {
            let mut buffer_struct = ARIB_STD_B25_BUFFER {
                data: null_mut(),
                size: 0,
            };

            let code = self.dec.as_ref().get(&mut buffer_struct);
            std::ptr::copy_nonoverlapping(
                buffer_struct.data as *const u8,
                buf.as_mut_ptr(),
                buffer_struct.size as usize,
            );
            (code, buffer_struct.size as usize)
        };

        match code {
            0 => Ok(sz),
            _ => {
                let err = AribB25DecoderError::from(code);
                eprintln!("{}", err);
                // if greater than 0, it means that the decoder emitted some warnings.
                // if less than 0, it means that the decoder emitted some errors.
                if code > 0 {
                    Ok(sz)
                } else {
                    Err(std::io::Error::new(std::io::ErrorKind::Other, err))
                }
            }
        }
    }
}
