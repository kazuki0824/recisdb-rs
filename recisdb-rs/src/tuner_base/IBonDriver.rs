#![allow(non_snake_case, non_camel_case_types, unused_allocation, dead_code)]
include!(concat!(env!("OUT_DIR"), "/BonDriver_binding.rs"));

use std::ptr::NonNull;
use cpp_utils::{DynamicCast, Ptr, MutPtr};

impl BonDriver {
    pub fn create<const BUF_SZ: usize>(&self) -> IBon<BUF_SZ> {
        let IBon1 = unsafe {
            let ptr = self.CreateBonDriver();
            NonNull::new(ptr).unwrap()
        };

        let (IBon2, IBon3) = unsafe {
            let ptr: MutPtr<IBonDriver> = MutPtr::from_raw(IBon1.as_ptr());
            let IBon2 = ptr.dynamic_cast_mut();
            let IBon3 = IBon2.dynamic_cast_mut();
            (
                NonNull::new(IBon2.as_mut_raw_ptr()),
                NonNull::new(IBon3.as_mut_raw_ptr()),
            )
        };

        IBon{
            0: IBon1,
            1: IBon2,
            2: IBon3,
            3: [0;BUF_SZ]
        }
    }
}

extern "C" {
    pub fn interface_check_2(i: *mut IBonDriver) -> *mut IBonDriver2;
}
extern "C" {
    pub fn interface_check_3(i: *mut IBonDriver2) -> *mut IBonDriver3;
}

impl DynamicCast<IBonDriver2> for IBonDriver
{
    unsafe fn dynamic_cast(ptr: Ptr<Self>) -> Ptr<IBonDriver2> {
        Ptr::from_raw(interface_check_2(ptr.as_raw_ptr() as *mut _))
    }

    unsafe fn dynamic_cast_mut(ptr: MutPtr<Self>) -> MutPtr<IBonDriver2> {
        MutPtr::from_raw(interface_check_2(ptr.as_mut_raw_ptr()) as *mut IBonDriver2)
    }
}
impl DynamicCast<IBonDriver3> for IBonDriver2
{
    unsafe fn dynamic_cast(ptr: Ptr<Self>) -> Ptr<IBonDriver3> {
        Ptr::from_raw(interface_check_3(ptr.as_raw_ptr() as *mut _))
    }

    unsafe fn dynamic_cast_mut(ptr: MutPtr<Self>) -> MutPtr<IBonDriver3> {
        MutPtr::from_raw(interface_check_3(ptr.as_mut_raw_ptr()) as *mut IBonDriver3)
    }
}

pub struct IBon<const SZ: usize>(pub(crate) NonNull<IBonDriver>, Option<NonNull<IBonDriver2>>, Option<NonNull<IBonDriver3>>, [u8;SZ]);
impl Drop for IBon<_>
{
    fn drop(&mut self) {
        self.2 = None;
        self.1 = None;
        self.Release();
    }
}

type E = crate::tuner_base::error::BonDriverError;
impl IBon<_>
{
    //automatically select which version to use, like https://github.com/DBCTRADO/LibISDB/blob/519f918b9f142b77278acdb71f7d567da121be14/LibISDB/Windows/Base/BonDriver.cpp#L175
    pub(crate) fn OpenTuner(&self) -> Result<(), E>
    {
        unsafe {
            let vt = self.0.as_ref().vtable_ as *mut _;
            if IBonDriver_OpenTuner(vt) != 0 { Ok(()) }
            else { Err(E::OpenError) }
        }
    }
    pub(crate) fn Release(&self)
    {
        unsafe {
            let vt = self.0.as_ref().vtable_ as *mut _;
            IBonDriver_Release(vt)
        }
    }
    pub(crate) fn SetChannel(&self, ch: u8) -> Result<(), E>
    {
        unsafe {
            let vt = self.0.as_ref().vtable_ as *mut _;
            if IBonDriver_SetChannel(vt, ch) != 0 { Ok(())}
            else { Err(E::TuneError)}
        }
    }
}