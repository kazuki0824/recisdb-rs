use cbc_mac::{CbcMac, Mac};
use cbc_mac::digest::MacError;
use cryptography_b25_00::feistel;
use tail_cbc::cipher;
use tail_cbc::cipher::{BlockCipher, InvalidLength, Key, KeyInit, KeySizeUser};
use tail_cbc::cipher::typenum::{U1, U8};
use crate::access_control::block00_structure::BlockSize00;

type KeySizeRound = U8;

pub fn verify_mac(mac: &[u8], text: &[u8], key: Key<Round00>) -> Result<(), MacError>
{
    let mut cbc_mac = <CbcMac<Round00> as Mac>::new(&key);
    cbc_mac.update(text);
    cbc_mac.verify_truncated_right(mac.into())
}

#[derive(Clone)]
pub struct Round00 {
    key: u64,
}
impl BlockCipher for Round00 {}
impl KeySizeUser for Round00 {
    type KeySize = KeySizeRound;
}
impl KeyInit for Round00 {
    fn new(key: &Key<Self>) -> Self {
        Self::new_from_slice(key).unwrap()
    }
    fn new_from_slice(key: &[u8]) -> Result<Self, InvalidLength> {
        match key.len() {
            8 => {
                let key = u64::from_le_bytes(key.try_into().unwrap());
                Ok(Self { key })
            },
            _ => Err(InvalidLength),
        }
    }
}
crate::impl_block_encdec!(
    Round00, BlockSize00, U1, cipher, block,
    encrypt: {
        let b = block.get_in();
        let left_key = (cipher.key >> 32) as u32;
        let right_key = cipher.key as u32;

        let result = feistel(u64::from_be_bytes(b[0..8].try_into().unwrap()), left_key, right_key, 3, 3);

        let block = block.get_out();
        block[0..8].copy_from_slice(&result.to_be_bytes());
    }
    decrypt: {
        let b = block.get_in();
        let left_key = (cipher.key >> 32) as u32;
        let right_key = cipher.key as u32;

        let result = feistel(u64::from_be_bytes(b[0..8].try_into().unwrap()), left_key, right_key, 3, 3);

        let block = block.get_out();
        block[0..8].copy_from_slice(&result.to_le_bytes());
    }
);

#[test]
fn test_verify_cbcmac_zero() {
    let key = 0x0001020304050607u64.to_le_bytes();
    let ciphertext = [0; 16];
    let mac = (0x191512e86730b0e7u64).to_be_bytes();
    assert!(verify_mac(&mac, &ciphertext, key.into()).is_ok());
}

#[test]
fn test_verify_cbcmac_unaligned_random_values() {
    let key = 0x0001020304050607u64.to_le_bytes();
    let ciphertext = [ 0xC6, 0x80, 0x20, 0x3A, 0x53, 0x1E, 0x5C, 0xC7,
        0xB6, 0x08, 0xAF, 0xA1, 0x2B, 0x19, 0x26, 0x8A,
        0x47, 0xE1, 0x86, 0x74, 0xE9, ];
    let mac = (0x6e4087f4ef22b5du64).to_be_bytes();
    assert!(verify_mac(&mac, &ciphertext, key.into()).is_ok());
}