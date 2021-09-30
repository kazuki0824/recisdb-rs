use std::cell::Cell;
use crate::access_control::types::{WorkingKey, EmmBody, EmmReceivingKeys};
use std::sync::mpsc::{Sender, Receiver};
use crate::utils::BlockConversionSolver40;

mod ffi;
pub mod types;

pub(crate) struct EcmKeyHolder {
    pub(crate) key_pair: Cell<WorkingKey>,
}

pub struct EmmVerifier
{
    inner: EmmReceivingKeys,
    inner_extended: BlockConversionSolver40
}
impl EmmVerifier
{
    pub fn new(keys: EmmReceivingKeys) -> Self
    {
        unimplemented!("")
    }
}

pub(crate) type EmmChannel = (Sender<EmmBody>, Receiver<EmmBody>);
