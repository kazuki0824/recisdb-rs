use log::{debug, error, warn};
use pin_project_lite::pin_project;
use std::io::{Error, ErrorKind, Read, Write};
use std::ptr::null_mut;
use std::ptr::NonNull;

use crate::bindings::arib_std_b25::{ARIB_STD_B25, ARIB_STD_B25_BUFFER, B_CAS_CARD};
use crate::bindings::error::AribB25DecoderError;

mod arib_std_b25;
mod error;

#[cfg(feature = "block00cbc")]
mod ffi;

extern "C" {
    #[cfg(feature = "prioritized_card_reader")]
    pub(crate) fn override_card_reader_name_pattern(
        name: *const ::std::os::raw::c_char,
    ) -> ::std::os::raw::c_int;
}

pin_project! {
    pub(crate) struct InnerDecoder {
        #[pin]
        pub dec: NonNull<ARIB_STD_B25>,
        #[pin]
        cas: Option<Box<B_CAS_CARD>>,
    }
    impl PinnedDrop for InnerDecoder {
        fn drop(this: Pin<&mut Self>) {
            //Release the decoder instance
            if let Some(cas) = this.get_mut().cas.take() {
                drop(cas)
            }

            debug!("InnerDecoder has been released.")
        }
    }
}

impl InnerDecoder {
    #[allow(unused_variables)]
    pub(crate) unsafe fn new(key: bool) -> Result<Self, Error> {
        let dec = arib_std_b25::create_arib_std_b25();

        #[cfg(feature = "block00cbc")]
        if key {
            return Self::new_with_key(dec);
        }
        Self::new_without_key(dec)
    }

    #[cfg(feature = "block00cbc")]
    unsafe fn new_with_key(dec: *mut ARIB_STD_B25) -> Result<Self, Error> {
        let mut cas = B_CAS_CARD::default();
        //Allocate private data inside B_CAS_CARD
        cas.initialize()?;
        let ret = Self {
            dec: NonNull::new(dec).unwrap(),
            cas: Some(Box::new(cas)),
        };
        ret.dec.as_ref().set_b_cas_card(ret.cas.as_ref().unwrap());
        Ok(ret)
    }
    unsafe fn new_without_key(dec: *mut ARIB_STD_B25) -> Result<Self, Error> {
        let cas = arib_std_b25::create_b_cas_card();
        if cas.is_null() {
            Err(Error::new(
                ErrorKind::Other,
                AribB25DecoderError::ARIB_STD_B25_ERROR_EMPTY_B_CAS_CARD,
            ))
        } else {
            // Initialize the CAS card
            (*cas).initialize()?;
            (*dec).set_b_cas_card(&*cas);
            Ok(Self {
                dec: NonNull::new(dec).unwrap(),
                cas: None,
            })
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
                    // suppress warning (The NOT_COMPLETE error is generated at the time of initial reception because of the specification)
                    // warn!("{}", err);
                    Ok(buf.len())
                } else {
                    error!("{}", err);
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
                // if greater than 0, it means that the decoder emitted some warnings.
                // if less than 0, it means that the decoder emitted some errors.
                if code > 0 {
                    warn!("{}", err);
                    Ok(())
                } else {
                    error!("{}", err);
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
            if buffer_struct.data.is_null() {
                (0, 0)
            } else {
                std::ptr::copy_nonoverlapping(
                    buffer_struct.data as *const u8,
                    buf.as_mut_ptr(),
                    buffer_struct.size as usize,
                );
                (code, buffer_struct.size as usize)
            }
        };

        match code {
            0 => Ok(sz),
            _ => {
                let err = AribB25DecoderError::from(code);
                // if greater than 0, it means that the decoder emitted some warnings.
                // if less than 0, it means that the decoder emitted some errors.
                if code > 0 {
                    warn!("{}", err);
                    Ok(sz)
                } else {
                    error!("{}", err);
                    Err(std::io::Error::new(std::io::ErrorKind::Other, err))
                }
            }
        }
    }
}
