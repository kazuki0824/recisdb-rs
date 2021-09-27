use crate::utils::WorkingKey;
use std::cell::Cell;

mod ffi;

pub(crate) struct EcmKeyHolder {
    pub key_pair: Cell<WorkingKey>,
}

#[repr(C)]
pub struct EmmBody;
