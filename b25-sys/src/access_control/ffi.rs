use std::io::Cursor;
use tail_cbc::cipher::KeyIvInit;
use cryptography_b25_00::expand_00;

pub type Block00CbcDec = tail_cbc::Decryptor<cryptography_b25_00::Block00>;

#[no_mangle]
pub extern "C" fn post_scramble_key(src: *const u8, len: usize, dst: *mut u8) {
    unsafe {
        let payload = {
            let recv = &*std::ptr::slice_from_raw_parts(src, len);
            recv.to_vec()
        };

        let key = KEYHOLDER.get_unchecked().key_pair.get();

        let ks = {
            let size = payload.len();
            if size < 19 {
                Err(())
            } else {
                let protocol = payload[0];
                let working_key_id = payload[2];
                let cipher = &payload[3..size - 1];
                // let k = if working_key_id { &expand_00(0x15f8c5bf840b6694u64, 0) };
                let k = expand_00(0x15f8c5bf840b6694u64, 0);
                let dec = Block00CbcDec::new(
                    k,
                    &0xfe27199919690911u64.swap_bytes().to_ne_bytes().into()
                );
                Ok((Vec::from(cipher), ))
            }
        };

        if let Ok(result) = ks {
            std::ptr::copy_nonoverlapping(result.as_ptr(), dst, result.len());
        }
    }
}

#[no_mangle]
pub extern "C" fn post_emm(src: *const u8, len: usize) {
    unsafe {
        let recv = &*std::ptr::slice_from_raw_parts(src, len);
        if let Some((tx, _rx)) = CHANNEL.get() {
            let raw_emm = recv.to_vec();

            if let Ok(contents) = format_emm(raw_emm) {
                tx.send(contents).unwrap();
            }
        }
    }
}
