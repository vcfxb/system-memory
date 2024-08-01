//! Windows specific implementation of getting memory info/status.

use core::mem::MaybeUninit;
use errno::Errno;
use windows_sys::Win32::System::SystemInformation::{GlobalMemoryStatusEx, MEMORYSTATUSEX};

/// Appropriately populate a [MEMORYSTATUSEX] object using the [GlobalMemoryStatusEx] syscall.
///
/// If [GlobalMemoryStatusEx] errors, return the error code from [GlobalMemoryStatusEx].
pub fn populate_mem_status() -> Result<MEMORYSTATUSEX, Errno> {
    // so assuming initialization from zeroed memory is safe (there are no references which would be null).
    let mut mem_status_ex: MaybeUninit<MEMORYSTATUSEX> = MaybeUninit::uninit();

    // Set the length field of the memory status struct as required before calling into the windows API.
    // SAFETY: We are writing to this pointer not reading it, so it should be safe.
    unsafe { (*mem_status_ex.as_mut_ptr()).dwLength = size_of::<MEMORYSTATUSEX>() as u32 };

    // Call the windows system API.
    let return_value = unsafe { GlobalMemoryStatusEx(mem_status_ex.as_mut_ptr()) };

    if return_value != 0 {
        // SAFETY: We assume that GlobalMemoryStatusEx properly initialized this instance.
        Ok(unsafe { mem_status_ex.assume_init() })
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
