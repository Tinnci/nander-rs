//! CH347 WCH Vendor DLL Transport (Windows)
//!
//! Provides direct communication with CH347 using Nanjing Qinheng (WCH)'s
//! official driver DLL (CH347DLLA64.DLL / CH347DLL.DLL).
//! This avoids the need to replace the driver with WinUSB via Zadig.

#[cfg(target_os = "windows")]
use crate::error::{Error, Result};
#[cfg(target_os = "windows")]
use std::ffi::c_void;
#[cfg(target_os = "windows")]
use std::sync::Arc;

#[cfg(target_os = "windows")]
mod win32 {
    use std::ffi::c_void;

    #[link(name = "kernel32")]
    extern "system" {
        pub fn LoadLibraryA(lpLibFileName: *const u8) -> *mut c_void;
        pub fn GetProcAddress(hModule: *mut c_void, lpProcName: *const u8) -> *mut c_void;
        pub fn FreeLibrary(hLibModule: *mut c_void) -> i32;
    }
}

#[cfg(target_os = "windows")]
type FnOpenDevice = unsafe extern "system" fn(i_index: u32) -> *mut c_void;
#[cfg(target_os = "windows")]
type FnCloseDevice = unsafe extern "system" fn(i_index: u32) -> i32;
#[cfg(target_os = "windows")]
type FnSetTimeout = unsafe extern "system" fn(i_index: u32, write_timeout: u32, read_timeout: u32) -> u32;
#[cfg(target_os = "windows")]
type FnWriteData = unsafe extern "system" fn(i_index: u32, buffer: *const u8, length: *mut u32) -> u32;
#[cfg(target_os = "windows")]
type FnReadData = unsafe extern "system" fn(i_index: u32, buffer: *mut u8, length: *mut u32) -> u32;
#[cfg(target_os = "windows")]
type FnGetDeviceInfor = unsafe extern "system" fn(i_index: u32, dev_info: *mut u8) -> u32;

#[cfg(target_os = "windows")]
pub struct WchDll {
    h_module: *mut c_void,
    pub open_device: FnOpenDevice,
    pub close_device: FnCloseDevice,
    pub set_timeout: FnSetTimeout,
    pub write_data: FnWriteData,
    pub read_data: FnReadData,
    pub get_device_infor: FnGetDeviceInfor,
}

#[cfg(target_os = "windows")]
unsafe impl Send for WchDll {}
#[cfg(target_os = "windows")]
unsafe impl Sync for WchDll {}

#[cfg(target_os = "windows")]
impl WchDll {
    pub fn load() -> Result<Arc<Self>> {
        let dll_names = [
            b"CH347DLLA64.DLL\0".as_ptr(),
            b"CH347DLL.DLL\0".as_ptr(),
        ];

        let mut h_mod = std::ptr::null_mut();
        for &name in &dll_names {
            unsafe {
                let m = win32::LoadLibraryA(name);
                if !m.is_null() {
                    h_mod = m;
                    break;
                }
            }
        }

        if h_mod.is_null() {
            return Err(Error::Other("Failed to load CH347DLLA64.DLL or CH347DLL.DLL".into()));
        }

        unsafe {
            macro_rules! get_proc {
                ($fn_type:ty, $name:expr) => {{
                    let p = win32::GetProcAddress(h_mod, $name.as_ptr());
                    if p.is_null() {
                        win32::FreeLibrary(h_mod);
                        return Err(Error::Other(
                            "Missing required symbol in CH347 DLL".into()
                        ));
                    }
                    let f: $fn_type = std::mem::transmute(p);
                    f
                }};
            }

            let open_device: FnOpenDevice = get_proc!(FnOpenDevice, b"CH347OpenDevice\0");
            let close_device: FnCloseDevice = get_proc!(FnCloseDevice, b"CH347CloseDevice\0");
            let set_timeout: FnSetTimeout = get_proc!(FnSetTimeout, b"CH347SetTimeout\0");
            let write_data: FnWriteData = get_proc!(FnWriteData, b"CH347WriteData\0");
            let read_data: FnReadData = get_proc!(FnReadData, b"CH347ReadData\0");
            let get_device_infor: FnGetDeviceInfor = get_proc!(FnGetDeviceInfor, b"CH347GetDeviceInfor\0");

            Ok(Arc::new(Self {
                h_module: h_mod,
                open_device,
                close_device,
                set_timeout,
                write_data,
                read_data,
                get_device_infor,
            }))
        }
    }
}

#[cfg(target_os = "windows")]
impl Drop for WchDll {
    fn drop(&mut self) {
        if !self.h_module.is_null() {
            unsafe {
                win32::FreeLibrary(self.h_module);
            }
        }
    }
}

// Dummy implementation for non-Windows platforms
#[cfg(not(target_os = "windows"))]
pub struct WchDll;
