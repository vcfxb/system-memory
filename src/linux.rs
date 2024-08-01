//! Linux specific implementation.

use core::{ffi::c_int, mem::MaybeUninit};
use errno::Errno;
use libc::sysinfo;

/// Get information about the host machine using [`sysinfo`](fn@sysinfo) or return [`errno`] if it fails.
pub fn populate_sysinfo() -> Result<sysinfo, Errno> {
    let mut sys_info: MaybeUninit<sysinfo> = MaybeUninit::uninit();

    // Call sysinfo syscall.
    let return_code: c_int = unsafe { sysinfo(sys_info.as_mut_ptr()) };

    if return_code < 0 {
        Err(errno::errno())
    } else {
        // SAFETY: Assume that the syscall properly initialized the instance.
        Ok(unsafe { sys_info.assume_init() })
    }
}

/// Get a properly populated [`sysinfo`](struct@sysinfo) or panic.
///
/// # Panics
/// - If the underlying [`sysinfo`](fn@sysinfo) call fails.
pub fn get_sysinfo() -> sysinfo {
    populate_sysinfo().expect("could not get sysinfo")
}
