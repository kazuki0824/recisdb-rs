use crate::access_control::EmmBody;
use crate::utils::BlockConversionSolver00;
use crate::{CHANNEL, KEYHOLDER};

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
            }
            else {
                let protocol = payload[0];
                let working_key_id = payload[2];
                let cipher = &payload[3..size - 1];
                let solver = BlockConversionSolver00::new(key, protocol);
                Ok(solver.convert(Vec::from(cipher), working_key_id))
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
            unimplemented!("Convert to EmmBody");
            tx.send(EmmBody); //TODO
        }
    }
}

