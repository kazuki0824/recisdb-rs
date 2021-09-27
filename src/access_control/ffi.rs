use crate::access_control::EmmBody;
use crate::utils::{BlockConversionSolver00, WorkingKey};
use crate::{CHANNEL, KEYHOLDER};

#[no_mangle]
pub extern "C" fn post_scramble_key(src: *const u8, len: usize, dst: *mut u8) {
    unsafe {
        let recv = &*std::ptr::slice_from_raw_parts(src, len);
        let recv = recv.to_vec();

        if let Ok(result) = proc_ecm(recv, KEYHOLDER.get_unchecked().key_pair.get()) {
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
            tx.send(EmmBody); //TODO
        }
    }
}

fn proc_ecm(mut payload: Vec<u8>, key: WorkingKey) -> Result<Vec<u8>, ()> {
    let size = payload.len();
    if size < 19 {
        return Err(());
    };

    let protocol = payload[0];
    let working_key_id = payload[2];
    let cipher = &mut payload[3..size - 1];

    let b = BlockConversionSolver00::new(key, protocol);
    let ks = b.convert(Vec::from(cipher), working_key_id);

    Ok(ks)
}
