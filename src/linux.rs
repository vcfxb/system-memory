//! Linux specific implementation.

use core::{ffi::c_int, mem};
use errno::Errno;
use libc::sysinfo;

/// Get information about the host machine using [`sysinfo`](fn@sysinfo) or return [`errno`] if it fails.
pub fn populate_sysinfo() -> Result<sysinfo, Errno> {
    // SAFETY: Sysinfo struct does not contain any reference/pointer types so populating from zeroed memory is safe.
    let mut sys_info: sysinfo = unsafe { mem::zeroed() };

    // Call sysinfo syscall.
    let return_code: c_int = unsafe { sysinfo(&mut sys_info as *mut _) };

    if return_code < 0 {
        Err(errno::errno())
    } else {
        Ok(sys_info)
    }
}

/// Get a properly populated [`sysinfo`] or panic.
///
/// # Panics
/// - If the underlying [`sysinfo`](fn@sysinfo) call fails.
pub fn get_sysinfo() -> sysinfo {
    populate_sysinfo().expect("could not get sysinfo")
}
