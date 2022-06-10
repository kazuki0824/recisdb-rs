use std::cell::Cell;
use std::io::{Read, Write};
use std::mem::ManuallyDrop;
use std::pin::Pin;
use std::ptr::null_mut;

use crate::bindings::arib_std_b25::{ARIB_STD_B25, ARIB_STD_B25_BUFFER, B_CAS_CARD};
use crate::bindings::error::AribB25DecoderError;
use crate::WorkingKey;

mod arib_std_b25;
mod error;
mod ffi;

pub(crate) struct InnerDecoder<'a> {
    pub dec: Pin<&'a mut ARIB_STD_B25>,
    cas: ManuallyDrop<B_CAS_CARD>,
    key: Cell<Option<WorkingKey>>,
}
impl InnerDecoder<'_> {
    pub(crate) unsafe fn new(key: Option<WorkingKey>) -> Result<Self, AribB25DecoderError> {
        let mut dec = arib_std_b25::create_arib_std_b25();

        // Clone the instance from the orignal that starts from the address created by create_arib_std_b25()
        // If the program crashed when this instance is freed, this code is the cause of the crash.
        match key {
            None => {
                let cas = arib_std_b25::create_b_cas_card();
                if cas.is_null() {
                    Err(AribB25DecoderError::ARIB_STD_B25_ERROR_EMPTY_B_CAS_CARD)
                } else {
                    Ok(Self {
                        dec: Pin::new_unchecked(&mut *dec),
                        cas: ManuallyDrop::new(*cas.clone()),
                        key: Cell::new(None),
                    })
                }
            }
            Some(key) => {
                let mut cas = B_CAS_CARD::default();
                //Allocate private data inside B_CAS_CARD
                cas.initialize();
                (*dec).set_b_cas_card(&mut cas);
                Ok(Self {
                    dec: Pin::new_unchecked(&mut *dec),
                    cas: ManuallyDrop::new(cas),
                    key: Cell::new(Some(key)),
                })
            }
        }
    }
}

impl Drop for InnerDecoder<'_> {
    fn drop(&mut self) {
        unsafe {
            //FIXME: Release the decoder instance
            //self.dec.get_unchecked_mut().release();
        }
    }
}

impl Write for InnerDecoder<'_> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let code = unsafe {
            let mut buffer_struct = ARIB_STD_B25_BUFFER {
                data: std::mem::transmute::<*const u8, *mut u8>(buf.as_ptr()),
                size: buf.len() as u32,
            };
            self.dec.put(&buffer_struct)
        };

        match code {
            0 => Ok(buf.len()),
            _ => {
                let err = AribB25DecoderError::from(code);
                eprintln!("{}", err);
                // if greater than 0, it means that the decoder emitted some warnings.
                // if less than 0, it means that the decoder emitted some errors.
                if code > 0 {
                    Ok(buf.len())
                } else {
                    Err(std::io::Error::new(std::io::ErrorKind::Other, err))
                }
            }
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        let code = self.dec.flush();

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

impl Read for InnerDecoder<'_> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let (code, sz) = unsafe {
            let mut buffer_struct = ARIB_STD_B25_BUFFER {
                data: null_mut(),
                size: 0,
            };

            let code = self.dec.get(&mut buffer_struct);
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
