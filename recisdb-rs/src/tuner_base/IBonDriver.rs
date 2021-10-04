#![allow(non_snake_case, non_camel_case_types, unused_allocation, dead_code)]

use std::ptr::NonNull;
use std::time::Duration;

use cpp_utils::{DynamicCast, MutPtr, Ptr};

include!(concat!(env!("OUT_DIR"), "/BonDriver_binding.rs"));

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
impl<const SZ: usize> Drop for IBon<SZ>
{
    fn drop(&mut self) {
        self.2 = None;
        self.1 = None;
        self.Release();
    }
}

type E = crate::tuner_base::error::BonDriverError;
impl<const SZ: usize> IBon<SZ>
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
    pub(crate) fn SetChannelBySpace(&self, space: u8, ch: u8) -> Result<(), E>
    {
        todo!()
    }
    pub(crate) fn WaitTsStream(&self, timeout: Duration) -> bool
    {
        unsafe {
            let vt = self.0.as_ref().vtable_ as *mut _;
            IBonDriver_WaitTsStream(vt, timeout.as_millis() as u32) != 0
        }
    }
    pub(crate) fn GetTsStream(&self) -> Result<(Vec<u8>, usize), E>
    {
        let (size, remaining) =
            unsafe {
                let mut size = 0 as u32;
                let mut remaining = 0 as u32;

                let vt = self.0.as_ref().vtable_ as *mut _;
                if IBonDriver_GetTsStream(vt, self.3.as_ptr() as *mut _, &mut size as *mut u32, &mut remaining as *mut u32) != 0
                {
                    Ok((size as usize, remaining as usize))
                }
                else { Err(E::GetTsError) }
            }?;
        let received = self.3[0..size].to_vec();
        Ok((received, remaining))
    }
}