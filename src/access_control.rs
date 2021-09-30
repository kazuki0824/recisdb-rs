use std::cell::Cell;
use crate::access_control::types::{WorkingKey, EmmBody, EmmReceivingKeys};
use std::sync::mpsc::{Sender, Receiver};
use crate::utils::BlockConversionSolver40;

mod ffi;
pub mod types;

pub(crate) struct EcmKeyHolder {
    pub(crate) key_pair: Cell<WorkingKey>,
}

pub struct EmmVerifier
{
    inner: EmmReceivingKeys,
    inner_extended: BlockConversionSolver40
}
impl EmmVerifier
{
    pub fn new(keys: EmmReceivingKeys) -> Self
    {
        unimplemented!("")
    }
}

pub(crate) type EmmChannel = (Sender<EmmBody>, Receiver<EmmBody>);

pub unsafe fn register_card_ids(&mut ids: Vec<i64>)
{
    let passing = B_CAS_ID{
        count: ids.len() as i32,
        data: ids.as_mut_ptr()
    };
    reg_id(passing);
}



extern "C" {
    fn reg_id(bCasId: B_CAS_ID) -> ::std::os::raw::c_int;
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
struct B_CAS_ID {
    pub data: *mut i64,
    pub count: i32,
}
#[test]
fn bindgen_test_layout_B_CAS_ID() {
    assert_eq!(
        ::std::mem::size_of::<B_CAS_ID>(),
        16usize,
        concat!("Size of: ", stringify!(B_CAS_ID))
    );
    assert_eq!(
        ::std::mem::align_of::<B_CAS_ID>(),
        8usize,
        concat!("Alignment of ", stringify!(B_CAS_ID))
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<B_CAS_ID>())).data as *const _ as usize },
        0usize,
        concat!(
        "Offset of field: ",
        stringify!(B_CAS_ID),
        "::",
        stringify!(data)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<B_CAS_ID>())).count as *const _ as usize },
        8usize,
        concat!(
        "Offset of field: ",
        stringify!(B_CAS_ID),
        "::",
        stringify!(count)
        )
    );
}