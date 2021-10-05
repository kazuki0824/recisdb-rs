#![allow(non_snake_case, non_camel_case_types, unused_allocation, dead_code)]

use std::ptr::NonNull;
use std::time::Duration;

use cpp_utils::{DynamicCast, MutPtr, Ptr};

include!(concat!(env!("OUT_DIR"), "/BonDriver_binding.rs"));

#[allow(clippy::all)]
mod ib1 {
    use super::{IBonDriver, BOOL, BYTE, DWORD};
    extern "C" {
        pub fn C_OpenTuner(b: *mut IBonDriver) -> BOOL;
        pub fn C_CloseTuner(b: *mut IBonDriver);
        pub fn C_SetChannel(b: *mut IBonDriver, bCh: BYTE) -> BOOL;
        pub fn C_GetSignalLevel(b: *mut IBonDriver) -> f32;
        pub fn C_WaitTsStream(b: *mut IBonDriver, dwTimeOut: DWORD) -> DWORD;
        pub fn C_GetReadyCount() -> DWORD;
        pub fn C_GetTsStream(
            b: *mut IBonDriver,
            pDst: *mut BYTE,
            pdwSize: *mut DWORD,
            pdwRemain: *mut DWORD,
        ) -> BOOL;
        pub fn C_GetTsStream2(
            b: *mut IBonDriver,
            ppDst: *mut *mut BYTE,
            pdwSize: *mut DWORD,
            pdwRemain: *mut DWORD,
        ) -> BOOL;
        pub fn C_PurgeTsStream(b: *mut IBonDriver);
        pub fn C_Release(b: *mut IBonDriver);
    }
}

impl BonDriver {
    pub fn create_interface<const BUF_SZ: usize>(&self) -> IBon<BUF_SZ> {
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

        IBon {
            1: IBon1,
            2: IBon2,
            3: IBon3,
            0: [0; BUF_SZ],
        }
    }
}

extern "C" {
    pub fn interface_check_2(i: *mut IBonDriver) -> *mut IBonDriver2;
    pub fn interface_check_3(i: *mut IBonDriver2) -> *mut IBonDriver3;
    pub fn interface_check_2_const(i: *const IBonDriver) -> *const IBonDriver2;
    pub fn interface_check_3_const(i: *const IBonDriver2) -> *const IBonDriver3;
}

impl DynamicCast<IBonDriver2> for IBonDriver {
    unsafe fn dynamic_cast(ptr: Ptr<Self>) -> Ptr<IBonDriver2> {
        Ptr::from_raw(interface_check_2_const(ptr.as_raw_ptr()))
    }

    unsafe fn dynamic_cast_mut(ptr: MutPtr<Self>) -> MutPtr<IBonDriver2> {
        MutPtr::from_raw(interface_check_2(ptr.as_mut_raw_ptr()))
    }
}
impl DynamicCast<IBonDriver3> for IBonDriver2 {
    unsafe fn dynamic_cast(ptr: Ptr<Self>) -> Ptr<IBonDriver3> {
        Ptr::from_raw(interface_check_3_const(ptr.as_raw_ptr()))
    }

    unsafe fn dynamic_cast_mut(ptr: MutPtr<Self>) -> MutPtr<IBonDriver3> {
        MutPtr::from_raw(interface_check_3(ptr.as_mut_raw_ptr()))
    }
}

pub struct IBon<const SZ: usize>(
    [u8; SZ],
    pub(crate) NonNull<IBonDriver>,
    pub(crate) Option<NonNull<IBonDriver2>>,
    pub(crate) Option<NonNull<IBonDriver3>>,
);
impl<const SZ: usize> Drop for IBon<SZ> {
    fn drop(&mut self) {
        self.3 = None;
        self.2 = None;
        self.Release();
    }
}

type E = crate::tuner_base::error::BonDriverError;
impl<const SZ: usize> IBon<SZ> {
    //automatically select which version to use, like https://github.com/DBCTRADO/LibISDB/blob/519f918b9f142b77278acdb71f7d567da121be14/LibISDB/Windows/Base/BonDriver.cpp#L175
    pub(crate) fn OpenTuner(&self) -> Result<(), E> {
        unsafe {
            let iface = self.1.as_ptr();
            if ib1::C_OpenTuner(iface) != 0 {
                Ok(())
            } else {
                Err(E::OpenError)
            }
        }
    }
    pub(crate) fn Release(&self) {
        unsafe {
            let iface = self.1.as_ptr();
            ib1::C_Release(iface)
        }
    }
    pub(crate) fn SetChannel(&self, ch: u8) -> Result<(), E> {
        unsafe {
            let iface = self.1.as_ptr();
            if ib1::C_SetChannel(iface, ch) != 0 {
                Ok(())
            } else {
                Err(E::TuneError)
            }
        }
    }
    pub(crate) fn SetChannelBySpace(&self, space: u8, ch: u8) -> Result<(), E> {
        todo!()
    }
    pub(crate) fn WaitTsStream(&self, timeout: Duration) -> bool {
        unsafe {
            let iface = self.1.as_ptr();
            ib1::C_WaitTsStream(iface, timeout.as_millis() as u32) != 0
        }
    }
    pub(crate) fn GetTsStream(&self) -> Result<(Vec<u8>, usize), E> {
        let (size, remaining) = unsafe {
            let mut size = 0_u32;
            let mut remaining = 0_u32;

            let iface = self.1.as_ptr();
            if ib1::C_GetTsStream(
                iface,
                self.0.as_ptr() as *mut _,
                &mut size as *mut u32,
                &mut remaining as *mut u32,
            ) != 0
            {
                Ok((size as usize, remaining as usize))
            } else {
                Err(E::GetTsError)
            }
        }?;
        let received = self.0[0..size].to_vec(); //Copying is necessary in order to avoid simultaneous access caused by next call
        Ok((received, remaining))
    }
}
