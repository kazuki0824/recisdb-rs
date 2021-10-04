use std::cell::Cell;
use std::sync::mpsc::{Receiver, Sender};

use byteorder::{BigEndian, ByteOrder};

use crate::access_control::types::{EmmBody, EmmDecryptedBody, EmmReceivingKeyPair, WorkingKey};
use crate::utils::BlockConversionSolver40;

mod ffi;
pub mod types;

pub(crate) struct EcmKeyHolder {
    pub(crate) key_pair: Cell<WorkingKey>,
}

pub struct EmmVerifier {
    inner_extended: Vec<BlockConversionSolver40>,
}
impl EmmVerifier {
    pub fn new(keys: Vec<EmmReceivingKeyPair>) -> Self {
        Self {
            inner_extended: keys
                .into_iter()
                .map(BlockConversionSolver40::new)
                .collect(),
        }
    }
    pub fn try_all_keys(&self, emm: EmmBody) -> Option<EmmDecryptedBody> {
        for b in &self.inner_extended {
            if emm.card_id == b.ex_key.0 {
                if let Some(result) = b.try_convert(emm.info.clone(), emm.protocol) {
                    return Some(EmmDecryptedBody {
                        card_id: emm.card_id,
                        protocol: emm.protocol,
                        broadcaster_group_id: result[0],
                        update_number: BigEndian::read_u16(&result[1..=2]),
                        expiration_date: BigEndian::read_u16(&result[3..=4]),
                        info: result[5..].into(),
                    });
                } else {
                    continue;
                }
            }
        }
        None
    }
}

pub(crate) type EmmChannel = (Sender<EmmBody>, Receiver<EmmBody>);
