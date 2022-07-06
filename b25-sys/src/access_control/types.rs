use std::fmt::{Display, Formatter};

pub type Block00CbcDec = tail_cbc::Decryptor<crate::access_control::block00_structure::Block00>;

pub struct EmmReceivingKeyPair {
    pub card_id: i64,
    pub key: u64,
}
pub(crate) struct EmmExtendedKeys(pub(crate) i64, pub(crate) [u32; 16]);

pub struct EmmBody {
    pub card_id: i64,
    pub protocol: u8,
    pub info: Vec<u8>,
}

impl Display for EmmBody {
    fn fmt(&self, _f: &mut Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

pub struct EmmDecryptedBody {
    pub card_id: i64,
    pub protocol: u8,
    pub broadcaster_group_id: u8,
    pub update_number: u16,
    pub expiration_date: u16,
    pub info: Vec<u8>,
}
