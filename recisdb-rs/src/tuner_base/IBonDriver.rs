#![allow(non_snake_case, non_camel_case_types, unused_allocation, dead_code)]


#[repr(C)]
pub struct IBonDriver__bindgen_vtable(::std::os::raw::c_void);
#[doc = " IBonDriver インターフェース"]
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct IBonDriver {
    pub vtable_: *const IBonDriver__bindgen_vtable,
}
pub type IBonDriver_BOOL = ::std::os::raw::c_int;
pub type IBonDriver_BYTE = u8;
pub type IBonDriver_DWORD = u32;
#[test]
fn bindgen_test_layout_IBonDriver() {
    assert_eq!(
        ::std::mem::size_of::<IBonDriver>(),
        8usize,
        concat!("Size of: ", stringify!(IBonDriver))
    );
    assert_eq!(
        ::std::mem::align_of::<IBonDriver>(),
        8usize,
        concat!("Alignment of ", stringify!(IBonDriver))
    );
}
extern "C" {
    #[link_name = "\u{1}?OpenTuner@IBonDriver@@UEAA?BHXZ"]
    pub fn IBonDriver_OpenTuner(this: *mut ::std::os::raw::c_void) -> IBonDriver_BOOL;
}
extern "C" {
    #[link_name = "\u{1}?CloseTuner@IBonDriver@@UEAAXXZ"]
    pub fn IBonDriver_CloseTuner(this: *mut ::std::os::raw::c_void);
}
extern "C" {
    #[link_name = "\u{1}?SetChannel@IBonDriver@@UEAA?BHE@Z"]
    pub fn IBonDriver_SetChannel(
        this: *mut ::std::os::raw::c_void,
        bCh: IBonDriver_BYTE,
    ) -> IBonDriver_BOOL;
}
extern "C" {
    #[link_name = "\u{1}?GetSignalLevel@IBonDriver@@UEAA?BMXZ"]
    pub fn IBonDriver_GetSignalLevel(this: *mut ::std::os::raw::c_void) -> f32;
}
extern "C" {
    #[link_name = "\u{1}?WaitTsStream@IBonDriver@@UEAA?BII@Z"]
    pub fn IBonDriver_WaitTsStream(
        this: *mut ::std::os::raw::c_void,
        dwTimeOut: IBonDriver_DWORD,
    ) -> IBonDriver_DWORD;
}
extern "C" {
    #[link_name = "\u{1}?GetReadyCount@IBonDriver@@UEAA?BIXZ"]
    pub fn IBonDriver_GetReadyCount(this: *mut ::std::os::raw::c_void) -> IBonDriver_DWORD;
}
extern "C" {
    #[link_name = "\u{1}?GetTsStream@IBonDriver@@UEAA?BHPEAEPEAI1@Z"]
    pub fn IBonDriver_GetTsStream(
        this: *mut ::std::os::raw::c_void,
        pDst: *mut IBonDriver_BYTE,
        pdwSize: *mut IBonDriver_DWORD,
        pdwRemain: *mut IBonDriver_DWORD,
    ) -> IBonDriver_BOOL;
}
extern "C" {
    #[link_name = "\u{1}?GetTsStream@IBonDriver@@UEAA?BHPEAPEAEPEAI1@Z"]
    pub fn IBonDriver_GetTsStream1(
        this: *mut ::std::os::raw::c_void,
        ppDst: *mut *mut IBonDriver_BYTE,
        pdwSize: *mut IBonDriver_DWORD,
        pdwRemain: *mut IBonDriver_DWORD,
    ) -> IBonDriver_BOOL;
}
extern "C" {
    #[link_name = "\u{1}?PurgeTsStream@IBonDriver@@UEAAXXZ"]
    pub fn IBonDriver_PurgeTsStream(this: *mut ::std::os::raw::c_void);
}
extern "C" {
    #[link_name = "\u{1}?Release@IBonDriver@@UEAAXXZ"]
    pub fn IBonDriver_Release(this: *mut ::std::os::raw::c_void);
}
#[doc = " IBonDriver2 インターフェース"]
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct IBonDriver2 {
    pub _base: IBonDriver,
}
pub type IBonDriver2_CharType = u16;
pub type IBonDriver2_LPCTSTR = *const IBonDriver2_CharType;
#[test]
fn bindgen_test_layout_IBonDriver2() {
    assert_eq!(
        ::std::mem::size_of::<IBonDriver2>(),
        8usize,
        concat!("Size of: ", stringify!(IBonDriver2))
    );
    assert_eq!(
        ::std::mem::align_of::<IBonDriver2>(),
        8usize,
        concat!("Alignment of ", stringify!(IBonDriver2))
    );
}
extern "C" {
    #[link_name = "\u{1}?GetTunerName@IBonDriver2@@UEAAPEB_SXZ"]
    pub fn IBonDriver2_GetTunerName(this: *mut ::std::os::raw::c_void) -> IBonDriver2_LPCTSTR;
}
extern "C" {
    #[link_name = "\u{1}?IsTunerOpening@IBonDriver2@@UEAA?BHXZ"]
    pub fn IBonDriver2_IsTunerOpening(this: *mut ::std::os::raw::c_void) -> IBonDriver_BOOL;
}
extern "C" {
    #[link_name = "\u{1}?EnumTuningSpace@IBonDriver2@@UEAAQEB_SI@Z"]
    pub fn IBonDriver2_EnumTuningSpace(
        this: *mut ::std::os::raw::c_void,
        dwSpace: IBonDriver_DWORD,
    ) -> IBonDriver2_LPCTSTR;
}
extern "C" {
    #[link_name = "\u{1}?EnumChannelName@IBonDriver2@@UEAAQEB_SII@Z"]
    pub fn IBonDriver2_EnumChannelName(
        this: *mut ::std::os::raw::c_void,
        dwSpace: IBonDriver_DWORD,
        dwChannel: IBonDriver_DWORD,
    ) -> IBonDriver2_LPCTSTR;
}
extern "C" {
    #[link_name = "\u{1}?SetChannel@IBonDriver2@@UEAA?BHII@Z"]
    pub fn IBonDriver2_SetChannel(
        this: *mut ::std::os::raw::c_void,
        dwSpace: IBonDriver_DWORD,
        dwChannel: IBonDriver_DWORD,
    ) -> IBonDriver_BOOL;
}
extern "C" {
    #[link_name = "\u{1}?GetCurSpace@IBonDriver2@@UEAA?BIXZ"]
    pub fn IBonDriver2_GetCurSpace(this: *mut ::std::os::raw::c_void) -> IBonDriver_DWORD;
}
extern "C" {
    #[link_name = "\u{1}?GetCurChannel@IBonDriver2@@UEAA?BIXZ"]
    pub fn IBonDriver2_GetCurChannel(this: *mut ::std::os::raw::c_void) -> IBonDriver_DWORD;
}
extern "C" {
    #[link_name = "\u{1}?Release@IBonDriver2@@UEAAXXZ"]
    pub fn IBonDriver2_Release(this: *mut ::std::os::raw::c_void);
}
#[doc = " IBonDriver3 インターフェース"]
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct IBonDriver3 {
    pub _base: IBonDriver2,
}
#[test]
fn bindgen_test_layout_IBonDriver3() {
    assert_eq!(
        ::std::mem::size_of::<IBonDriver3>(),
        8usize,
        concat!("Size of: ", stringify!(IBonDriver3))
    );
    assert_eq!(
        ::std::mem::align_of::<IBonDriver3>(),
        8usize,
        concat!("Alignment of ", stringify!(IBonDriver3))
    );
}
extern "C" {
    #[link_name = "\u{1}?GetTotalDeviceNum@IBonDriver3@@UEAA?BIXZ"]
    pub fn IBonDriver3_GetTotalDeviceNum(this: *mut ::std::os::raw::c_void) -> IBonDriver_DWORD;
}
extern "C" {
    #[link_name = "\u{1}?GetActiveDeviceNum@IBonDriver3@@UEAA?BIXZ"]
    pub fn IBonDriver3_GetActiveDeviceNum(this: *mut ::std::os::raw::c_void) -> IBonDriver_DWORD;
}
extern "C" {
    #[link_name = "\u{1}?SetLnbPower@IBonDriver3@@UEAA?BHH@Z"]
    pub fn IBonDriver3_SetLnbPower(
        this: *mut ::std::os::raw::c_void,
        bEnable: IBonDriver_BOOL,
    ) -> IBonDriver_BOOL;
}
extern "C" {
    #[link_name = "\u{1}?Release@IBonDriver3@@UEAAXXZ"]
    pub fn IBonDriver3_Release(this: *mut ::std::os::raw::c_void);
}
extern crate libloading;
extern crate cpp_utils;

use std::ptr::NonNull;
use self::cpp_utils::{DynamicCast, Ptr, MutPtr};

pub struct BonDriver {
    __library: ::libloading::Library,
    pub CreateBonDriver: unsafe extern "C" fn() -> *mut IBonDriver,
}
impl BonDriver {
    pub unsafe fn new<P>(path: P) -> Result<Self, ::libloading::Error>
    where
        P: AsRef<::std::ffi::OsStr>,
    {
        let library = ::libloading::Library::new(path)?;
        Self::from_library(library)
    }
    pub unsafe fn from_library<L>(library: L) -> Result<Self, ::libloading::Error>
    where
        L: Into<::libloading::Library>,
    {
        let __library = library.into();
        let CreateBonDriver = __library.get(b"CreateBonDriver\0").map(|sym| *sym)?;
        Ok(BonDriver {
            __library,
            CreateBonDriver,
        })
    }
    pub unsafe fn CreateBonDriver(&self) -> *mut IBonDriver {
        (self.CreateBonDriver)()
    }
}
