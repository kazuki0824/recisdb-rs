use crate::bindings::arib_std_b25::{ARIB_STD_B25, ARIB_STD_B25_BUFFER, B_CAS_CARD};
use crate::bindings::error::AribB25DecoderError;
use crate::WorkingKey;
use std::cell::Cell;
use std::ffi::c_void;
use std::io::{Read, Write};
use std::mem::ManuallyDrop;
use std::ptr::{null_mut, NonNull};

mod arib_std_b25;
mod error;
mod ffi;

pub(crate) struct InnerDecoder {
    pub dec: NonNull<ARIB_STD_B25>,
    cas: ManuallyDrop<B_CAS_CARD>,
    key: Cell<Option<WorkingKey>>,
}
impl InnerDecoder {
    pub(crate) unsafe fn new(key: Option<WorkingKey>) -> Result<Self, AribB25DecoderError> {
        let mut dec = NonNull::new(arib_std_b25::create_arib_std_b25()).unwrap();

        // Clone the instance from the orignal that starts from the address created by create_arib_std_b25()
        // If the program crashed when this instance is freed, this code is the cause of the crash.
        match key {
            None => {
                let cas = arib_std_b25::create_b_cas_card();
                if cas.is_null() {
                    Err(AribB25DecoderError::ARIB_STD_B25_ERROR_EMPTY_B_CAS_CARD)
                } else {
                    Ok(Self {
                        dec,
                        cas: ManuallyDrop::new(*cas.clone()),
                        key: Cell::new(None),
                    })
                }
            }
            Some(key) => {
                let mut cas = B_CAS_CARD::default();
                //Allocate private data inside B_CAS_CARD
                cas.initialize();
                dec.as_mut().set_b_cas_card.unwrap()(dec.as_ptr() as *mut _, &mut cas);
                Ok(Self {
                    dec,
                    cas: ManuallyDrop::new(cas),
                    key: Cell::new(Some(key)),
                })
            }
        }
    }
}

impl Drop for InnerDecoder {
    fn drop(&mut self) {
        unsafe {
            self.dec.as_mut().release.unwrap()(self.dec.as_ptr() as *mut c_void);
        }
    }
}

impl Write for InnerDecoder {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let code = unsafe {
            let put = self.dec.as_mut().put.unwrap();

            let mut buffer_struct = ARIB_STD_B25_BUFFER {
                data: std::mem::transmute::<*const u8, *mut u8>(buf.as_ptr()),
                size: buf.len() as u32,
            };

            put(
                &mut self.dec as *mut _ as *mut c_void,
                &mut buffer_struct as *mut ARIB_STD_B25_BUFFER,
            )
        };
        match code {
            0 => Ok(buf.len()),
            _ => {
                let err = AribB25DecoderError::from(code);
                Err(std::io::Error::new(std::io::ErrorKind::Other, err))
            }
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        let code = unsafe {
            let flush = self.dec.as_mut().flush.unwrap();
            flush(&mut self.dec as *mut _ as *mut c_void)
        };
        match code {
            0 => Ok(()),
            _ => {
                let err = AribB25DecoderError::from(code);
                Err(std::io::Error::new(std::io::ErrorKind::Other, err))
            }
        }
    }
}

impl Read for InnerDecoder {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let (code, sz) = unsafe {
            let get = self.dec.as_mut().get.unwrap();

            let mut buffer_struct = ARIB_STD_B25_BUFFER {
                data: null_mut(),
                size: 0,
            };

            let code = get(
                &mut self.dec as *mut _ as *mut c_void,
                &mut buffer_struct as *mut ARIB_STD_B25_BUFFER,
            );
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
                Err(std::io::Error::new(std::io::ErrorKind::Other, err))
            }
        }
    }
}
