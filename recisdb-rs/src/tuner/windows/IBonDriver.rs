#![allow(non_snake_case, non_camel_case_types, unused_allocation, dead_code)]

use std::io;
use std::ptr::NonNull;
use std::time::Duration;

use cpp_utils::{DynamicCast, MutPtr, Ptr};

include!(concat!(env!("OUT_DIR"), "/BonDriver_binding.rs"));

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
    use super::{IBonDriver2, BOOL, DWORD, LPCTSTR};

    extern "C" {
        //IBon2
        pub fn C_EnumTuningSpace(b: *mut IBonDriver2, dwSpace: DWORD) -> LPCTSTR;
        pub fn C_EnumChannelName2(b: *mut IBonDriver2, dwSpace: DWORD, dwChannel: DWORD)
            -> LPCTSTR;
        pub fn C_SetChannel2(b: *mut IBonDriver2, dwSpace: DWORD, dwChannel: DWORD) -> BOOL;
    }
}

mod ib3 {
    use crate::tuner::windows::IBonDriver::{IBonDriver3, BOOL};

    extern "C" {
        pub fn C_SetLnbPower(b: *mut IBonDriver3, bEnable: BOOL) -> BOOL;
    }
}

mod ib_utils {
    use super::{IBonDriver, IBonDriver2, IBonDriver3};

    extern "C" {
        pub(crate) fn interface_check_2(i: *mut IBonDriver) -> *mut IBonDriver2;
        pub(crate) fn interface_check_3(i: *mut IBonDriver2) -> *mut IBonDriver3;
        pub(crate) fn interface_check_2_const(i: *const IBonDriver) -> *const IBonDriver2;
        pub(crate) fn interface_check_3_const(i: *const IBonDriver2) -> *const IBonDriver3;
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
            let slice = std::slice::from_raw_parts(ptr, len);
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
    pub fn create_interface(&self) -> IBon {
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

        let version = match (IBon2, IBon3) {
            (None, None) => 1,
            (Some(_), None) => 2,
            (Some(_), Some(_)) => 3,
            _ => 0,
        };

        IBon(version, IBon1, IBon2, IBon3)
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

pub struct IBon(
    u8,
    pub(crate) NonNull<IBonDriver>,
    pub(crate) Option<NonNull<IBonDriver2>>,
    pub(crate) Option<NonNull<IBonDriver3>>,
);

impl Drop for IBon {
    fn drop(&mut self) {
        self.3 = None;
        self.2 = None;
        self.Release();
    }
}

type E = crate::tuner::error::BonDriverError;

impl IBon {
    //automatically select which version to use, like https://github.com/DBCTRADO/LibISDB/blob/519f918b9f142b77278acdb71f7d567da121be14/LibISDB/Windows/Base/BonDriver.cpp#L175
    //IBon1
    pub(crate) fn OpenTuner(&self) -> Result<(), std::io::Error> {
        unsafe {
            let iface = self.1.as_ptr();
            if ib1::C_OpenTuner(iface) != 0 {
                Ok(())
            } else {
                Err(io::Error::new(
                    io::ErrorKind::ConnectionRefused,
                    E::OpenError,
                ))
            }
        }
    }
    pub(crate) fn Release(&self) {
        unsafe {
            let iface = self.1.as_ptr();
            ib1::C_Release(iface)
        }
    }
    pub(crate) fn SetChannel(&self, ch: u8) -> Result<(), io::Error> {
        unsafe {
            let iface = self.1.as_ptr();
            if ib1::C_SetChannel(iface, ch) != 0 {
                Ok(())
            } else {
                Err(io::Error::new(
                    io::ErrorKind::AddrNotAvailable,
                    E::TuneError(ch),
                ))
            }
        }
    }
    pub(crate) fn WaitTsStream(&self, timeout: Duration) -> bool {
        unsafe {
            let iface = self.1.as_ptr();
            ib1::C_WaitTsStream(iface, timeout.as_millis() as u32) != 0
        }
    }
    pub(crate) fn GetTsStream<'a>(
        &self,
        buf: &'a mut [u8],
    ) -> Result<(&'a [u8], usize), io::Error> {
        let mut size = 0_u32;
        let mut remaining = 0_u32;

        let iface = self.1.as_ptr();
        unsafe {
            if ib1::C_GetTsStream(
                iface,
                buf.as_mut_ptr(),
                &mut size as *mut u32,
                &mut remaining as *mut u32,
            ) != 0
            {
                Ok((&buf[..size as usize], remaining as usize))
            } else {
                Err(io::Error::new(io::ErrorKind::UnexpectedEof, E::GetTsError))
            }
        }
    }
    pub(crate) fn GetSignalLevel(&self) -> Result<f32, io::Error> {
        let iface = self.1.as_ptr();
        return Ok(unsafe { ib1::C_GetSignalLevel(iface) });
    }
    //IBon2
    pub(crate) fn SetChannelBySpace(&self, space: u32, ch: u32) -> Result<(), io::Error> {
        unsafe {
            let iface = self.2.unwrap().as_ptr();
            if ib2::C_SetChannel2(iface, space, ch) != 0 {
                Ok(())
            } else {
                Err(io::Error::new(
                    io::ErrorKind::AddrNotAvailable,
                    E::InvalidSpaceChannel(space, ch),
                ))
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
    // IBon3
    pub(crate) fn SetLnbPower(&self, bEnable: BOOL) -> Result<(), io::Error> {
        unsafe {
            let iface = self.3.unwrap().as_ptr();
            if ib3::C_SetLnbPower(iface, bEnable) != 0 {
                Ok(())
            } else {
                Err(io::Error::new(io::ErrorKind::Unsupported, E::LnbError))
            }
        }
    }
}
