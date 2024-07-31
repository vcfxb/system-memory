//! Windows specific implementation of getting memory info/status.

use errno::Errno;
use windows_sys::Win32::System::SystemInformation::{GlobalMemoryStatusEx, MEMORYSTATUSEX};
use core::mem;

/// Appropriately populate a [MEMORYSTATUSEX] object using the [GlobalMemoryStatusEx] syscall.
/// 
/// If [GlobalMemoryStatusEx] errors, return the error code from [GlobalMemoryStatusEx].
pub fn populate_mem_status() -> Result<MEMORYSTATUSEX, Errno> {
    // SAFETY: All of the fields of this struct are unsigned integers of various sizes,
    // so assuming initialization from zeroed memory is safe (there are no references which would be null).
    let mut mem_status_ex: MEMORYSTATUSEX = unsafe { mem::zeroed() };

    // Set the length field of the memory status struct as required before calling into the windows API.
    mem_status_ex.dwLength = size_of::<MEMORYSTATUSEX>() as u32;

    // Call the windows system API.
    let return_value = unsafe { GlobalMemoryStatusEx(&mut mem_status_ex as *mut _) };

    if return_value != 0 {
        Ok(mem_status_ex)
    } else {
        Err(errno::errno())
    }
}

/// Populate a [MEMORYSTATUSEX] properly or panic.
/// 
/// # Panics
/// - If the underlying system call to windows returns an error.
pub fn mem_status() -> MEMORYSTATUSEX {
    populate_mem_status().expect("could not get system memory status")
}
