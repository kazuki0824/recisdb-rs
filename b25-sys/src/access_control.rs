use crate::access_control::block00_mac::verify_mac;
use crate::access_control::block00_structure::{expand_00, Block00};
use crate::access_control::types::Block00CbcDec;
use std::ops::Deref;
use tail_cbc::cipher::{Key, KeyIvInit};
use tail_cbc::UnalignedBytesDecryptMut;

mod block00_mac;
mod block00_structure;
mod block40_structure;
mod macros;
pub mod types;

pub(crate) fn select_key_by_auth(payload: &mut [u8]) -> Option<Key<Block00>> {
    let size = payload.len();

    let mut temp = payload.to_vec();
    let _protocol = payload[0];
    let _broadcaster_group_id = payload[1];
    let working_key_id = payload[2];
    let encrypted_part = &mut payload[3..];

    let mut ret = None;
    for k in if working_key_id % 2 == 1 {
        crate::KEY1.lock()
    } else {
        crate::KEY0.lock()
    }
    .unwrap()
    .deref()
    {
        let expanded_key = expand_00(*k, 0);
        let mut dec =
            Block00CbcDec::new(&expanded_key, &0xfe27199919690911u64.to_be_bytes().into());
        dec.decrypt_bytes_b2b_mut(encrypted_part, &mut temp[3..])
            .expect("decryption failed");

        //mac is the last 4 bytes of the payload
        let (content, mac) = temp.split_at_mut(size - 4);

        if verify_mac(mac, content, k.to_le_bytes().into()).is_ok() {
            encrypted_part.copy_from_slice(&temp[3..]);
            ret = Some(expanded_key);
            break;
        }
    }
    ret
}
