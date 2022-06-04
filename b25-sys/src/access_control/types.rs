use std::fmt::{Display, Formatter};

#[derive(Copy, Clone)]
pub struct WorkingKey(pub u64, pub u64);
impl WorkingKey {
    pub const DEFAULT: Self = Self(0x8d8206c62eb1410d, 0x15f8c5bf840b6694);
}

pub type Block00CbcDec = tail_cbc::Decryptor<cryptography_b25_00::Block00>;

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
