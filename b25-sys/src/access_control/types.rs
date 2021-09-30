#[derive(Copy, Clone)]
pub struct WorkingKey(pub u64, pub u64);

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

pub struct EmmDecryptedBody {
    pub card_id: i64,
    pub protocol: u8,
    pub broadcaster_group_id: u8,
    pub update_number: u16,
    pub expiration_date: u16,
    pub info: Vec<u8>,
}
