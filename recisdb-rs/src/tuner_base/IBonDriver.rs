#![allow(non_snake_case, non_camel_case_types, unused_allocation, dead_code)]

use std::ptr::NonNull;
use std::time::Duration;

use cpp_utils::{DynamicCast, MutPtr, Ptr};

use crate::channels::ChannelSpace;

include!(concat!(env!("OUT_DIR"), "/BonDriver_binding.rs"));

#[allow(clippy::all)]
mod ib1 {
    use super::{IBonDriver, BOOL, BYTE, DWORD};

    extern "C" {
        //IBon1
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
        //Common
        pub fn CreateBonDriver() -> *mut IBonDriver;
    }
}

mod ib2 {
    use super::{IBonDriver, IBonDriver2, BOOL, BYTE, DWORD, LPCTSTR};

    extern "C" {
        //IBon2
        pub fn C_EnumTuningSpace(b: *mut IBonDriver2, dwSpace: DWORD) -> LPCTSTR;
        pub fn C_EnumChannelName2(b: *mut IBonDriver2, dwSpace: DWORD, dwChannel: DWORD)
            -> LPCTSTR;
        pub fn C_SetChannel2(b: *mut IBonDriver2, dwSpace: DWORD, dwChannel: DWORD) -> BOOL;

    }
}

mod ib_utils {
    use super::{IBonDriver, IBonDriver2, IBonDriver3};

    extern "C" {
        pub(super) fn interface_check_2(i: *mut IBonDriver) -> *mut IBonDriver2;
        pub(super) fn interface_check_3(i: *mut IBonDriver2) -> *mut IBonDriver3;
        pub(super) fn interface_check_2_const(i: *const IBonDriver) -> *const IBonDriver2;
        pub(super) fn interface_check_3_const(i: *const IBonDriver2) -> *const IBonDriver3;
    }
    #[cfg(target_os = "windows")]
    pub(crate) fn from_wide_ptr(ptr: *const u16) -> Option<String> {
        // use std::ffi::OsString;
        // use std::os::windows::ffi::OsStringExt;
        // TODO: Still unstable. When displaying, it's better to use OsString.
        if ptr.is_null() {
            return None;
        }
        unsafe {
            let len = (0..std::isize::MAX)
                .position(|i| *ptr.offset(i) == 0)
                .unwrap();
            if len == 0 {
                return None;
            }
            let slice = std::slice::from_raw_parts(ptr, len as usize);
            // let os = OsString::from_wide(slice);
            // os.into_string().ok()
            String::from_utf16(slice).ok()
        }
    }
    #[cfg(target_os = "linux")]
    pub(crate) fn from_wide_ptr(_ptr: *const u16) -> Option<String> {
        None
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

impl DynamicCast<IBonDriver2> for IBonDriver {
    unsafe fn dynamic_cast(ptr: Ptr<Self>) -> Ptr<IBonDriver2> {
        Ptr::from_raw(ib_utils::interface_check_2_const(ptr.as_raw_ptr()))
    }

    unsafe fn dynamic_cast_mut(ptr: MutPtr<Self>) -> MutPtr<IBonDriver2> {
        MutPtr::from_raw(ib_utils::interface_check_2(ptr.as_mut_raw_ptr()))
    }
}
impl DynamicCast<IBonDriver3> for IBonDriver2 {
    unsafe fn dynamic_cast(ptr: Ptr<Self>) -> Ptr<IBonDriver3> {
        Ptr::from_raw(ib_utils::interface_check_3_const(ptr.as_raw_ptr()))
    }

    unsafe fn dynamic_cast_mut(ptr: MutPtr<Self>) -> MutPtr<IBonDriver3> {
        MutPtr::from_raw(ib_utils::interface_check_3(ptr.as_mut_raw_ptr()))
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
    //IBon1
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
                Err(E::TuneError(ch))
            }
        }
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
    //IBon2
    pub(crate) fn SetChannelBySpace(&self, space: u32, ch: u32) -> Result<(), E> {
        unsafe {
            let iface = self.2.unwrap().as_ptr();
            if ib2::C_SetChannel2(iface, space, ch) != 0 {
                Ok(())
            } else {
                Err(E::InvalidSpaceChannel(space, ch))
            }
        }
    }
    pub(crate) fn EnumTuningSpace(&self, space: u32) -> Option<String> {
        unsafe {
            let iface = self.2.unwrap().as_ptr();
            let returned = ib2::C_EnumTuningSpace(iface, space);
            ib_utils::from_wide_ptr(returned)
        }
    }
    pub(crate) fn EnumChannelName(&self, space: u32, ch: u32) -> Option<String> {
        unsafe {
            let iface = self.2.unwrap().as_ptr();
            let returned = ib2::C_EnumChannelName2(iface, space, ch);
            ib_utils::from_wide_ptr(returned)
        }
    }
}
