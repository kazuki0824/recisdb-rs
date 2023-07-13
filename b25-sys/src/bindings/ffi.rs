use std::marker::PhantomPinned;
use std::ptr::null_mut;

use crate::access_control::select_key_by_auth;
use log::{debug, warn};

use crate::bindings::arib_std_b25::{
    wchar_t, B_CAS_CARD, B_CAS_CARD_PRIVATE_DATA, B_CAS_ECM_RESULT, B_CAS_ID, B_CAS_INIT_STATUS,
    B_CAS_PWR_ON_CTRL, B_CAS_PWR_ON_CTRL_INFO,
};

// Overrides the functions of the struct `B_CAS_CARD`

#[no_mangle]
unsafe extern "C" fn release(bcas: *mut ::std::os::raw::c_void) {
    //free private data manually
    let _ = Box::from_raw((*(bcas as *mut B_CAS_CARD)).private_data as *mut B_CAS_CARD_PRIVATE_DATA);
}

const DEFAULT_NAME: &str = "b25-sys";
impl Default for B_CAS_CARD_PRIVATE_DATA {
    fn default() -> Self {
        B_CAS_CARD_PRIVATE_DATA {
            mng: 0,
            card: 0,
            pool: null_mut(),
            reader: DEFAULT_NAME.as_ptr() as *const wchar_t,
            sbuf: null_mut(),
            rbuf: null_mut(),
            stat: B_CAS_INIT_STATUS {
                system_key: [
                    0x36, 0x31, 0x04, 0x66, 0x4b, 0x17, 0xea, 0x5c, 0x32, 0xdf, 0x9c, 0xf5, 0xc4,
                    0xc3, 0x6c, 0x1b, 0xec, 0x99, 0x39, 0x21, 0x68, 0x9d, 0x4b, 0xb7, 0xb7, 0x4e,
                    0x40, 0x84, 0x0d, 0x2e, 0x7d, 0x98,
                ],
                init_cbc: [0xfe, 0x27, 0x19, 0x99, 0x19, 0x69, 0x09, 0x11],
                bcas_card_id: 0xfe2719991969091,
                card_status: 0,
                ca_system_id: 5,
            },
            id: B_CAS_ID {
                data: &mut [0i64; 1] as *mut _,
                count: 1,
            },
            id_max: 0,
            pwc: B_CAS_PWR_ON_CTRL_INFO {
                data: &mut B_CAS_PWR_ON_CTRL {
                    s_yy: 0,
                    s_mm: 0,
                    s_dd: 0,
                    l_yy: 0,
                    l_mm: 0,
                    l_dd: 0,
                    hold_time: 0,
                    broadcaster_group_id: 0,
                    network_id: 0,
                    transport_id: 0,
                },
                count: 0,
            },
            pwc_max: 0,
        }
    }
}
#[no_mangle]
unsafe extern "C" fn init(bcas: *mut ::std::os::raw::c_void) -> ::std::os::raw::c_int {
    (*(bcas as *mut B_CAS_CARD)).private_data =
        Box::into_raw(Box::new(B_CAS_CARD_PRIVATE_DATA::default())) as *mut _;
    0
}

#[no_mangle]
unsafe extern "C" fn get_init_status(
    bcas: *mut ::std::os::raw::c_void,
    stat: *mut B_CAS_INIT_STATUS,
) -> ::std::os::raw::c_int {
    let prv = (*(bcas as *mut B_CAS_CARD)).private_data as *mut B_CAS_CARD_PRIVATE_DATA;
    *stat = (*prv).stat; //Copy the status
    0
}

#[no_mangle]
unsafe extern "C" fn proc_ecm(
    _bcas: *mut ::std::os::raw::c_void,
    dst: *mut B_CAS_ECM_RESULT,
    src: *mut u8,
    len: ::std::os::raw::c_int,
) -> ::std::os::raw::c_int {
    let mut payload = {
        let recv = &*std::ptr::slice_from_raw_parts(src, len as usize);
        recv.to_vec()
    };

    let ks = {
        let size = payload.len();
        if size < 19 {
            Err(())
        } else {
            match select_key_by_auth(&mut payload) {
                Some(key) => {
                    debug!("Selected Kw= {:?}", key);
                    Ok(&payload[3..19])
                }
                None => {
                    warn!("No valid key found");
                    Err(())
                }
            }
        }
    };

    if let Ok(result) = ks {
        std::ptr::copy_nonoverlapping(
            result.as_ptr(),
            (*dst).scramble_key.as_mut_ptr(),
            result.len(),
        );
    }
    (*dst).return_code = 0x0800;
    return 0;
}

#[no_mangle]
unsafe extern "C" fn get_id(
    bcas: *mut ::std::os::raw::c_void,
    dst: *mut B_CAS_ID,
) -> ::std::os::raw::c_int {
    let prv = (*(bcas as *mut B_CAS_CARD)).private_data as *mut B_CAS_CARD_PRIVATE_DATA;
    (*dst).data = (*prv).id.data;
    (*dst).count = (*prv).id.count;
    0
}

unsafe extern "C" fn get_pwr_on_ctrl(
    bcas: *mut ::std::os::raw::c_void,
    dst: *mut B_CAS_PWR_ON_CTRL_INFO,
) -> ::std::os::raw::c_int {
    let prv = (*(bcas as *mut B_CAS_CARD)).private_data as *mut B_CAS_CARD_PRIVATE_DATA;
    (*dst).data = (*prv).pwc.data;
    (*dst).count = (*prv).pwc.count;
    0
}

#[no_mangle]
unsafe extern "C" fn proc_emm(
    bcas: *mut ::std::os::raw::c_void,
    src: *mut u8,
    len: ::std::os::raw::c_int,
) -> ::std::os::raw::c_int {
    0
}

impl Default for B_CAS_CARD {
    fn default() -> Self {
        B_CAS_CARD {
            private_data: null_mut(),
            release: Some(release),
            init: Some(init),
            get_init_status: Some(get_init_status),
            get_id: Some(get_id),
            get_pwr_on_ctrl: Some(get_pwr_on_ctrl),
            proc_ecm: Some(proc_ecm),
            proc_emm: Some(proc_emm),
            _pinned: PhantomPinned,
        }
    }
}
