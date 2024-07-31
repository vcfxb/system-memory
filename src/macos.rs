//! Mac OS specific implementation.

use core::ptr;
use core::{ffi::c_void, mem};
use errno::Errno;
use libc::{
    host_info64_t, host_statistics64, mach_host_self, mach_task_self, sysconf, sysctlbyname,
    vm_statistics64, HOST_VM_INFO64, HOST_VM_INFO64_COUNT, KERN_SUCCESS, _SC_PAGESIZE,
};
use mach::mach_port::mach_port_deallocate;

/// Get the page size using [`sysconf`] if possible.
pub fn page_size() -> Result<u64, Option<Errno>> {
    // Set errno before the call, since sysconf will not modify it if the operation is unsupported.
    errno::set_errno(Errno(0));

    // Call into sysconf to get the page size.
    let return_value = unsafe { sysconf(_SC_PAGESIZE) };

    if return_value < 0 {
        let error_code = errno::errno().0;

        // sysconf only updates errno in some cases.
        if error_code != 0 {
            Err(Some(Errno(error_code)))
        } else {
            Err(None)
        }
    } else {
        Ok(return_value as u64)
    }
}

/// Attempt to get this system's total physical memory using [`sysctlbyname`]. On error, return the code returned by
/// [`errno`].
pub fn try_get_total_physical_memory() -> Result<u64, Errno> {
    let name = c"hw.memsize";
    let mut memsize = 0u64;
    let mut memsize_len = size_of_val(&memsize);

    // SAFETY: We're calling into sysctlbyname with parameters set properly according to
    // https://developer.apple.com/documentation/kernel/1387446-sysctlbyname
    let return_value = unsafe {
        sysctlbyname(
            name.as_ptr(),
            &mut memsize as *mut _ as *mut c_void,
            &mut memsize_len,
            ptr::null_mut(),
            0,
        )
    };

    if return_value == 0 {
        Ok(memsize)
    } else {
        Err(errno::errno())
    }
}

/// Get the total physical memory on the host system.
///
/// # Panics
/// - Panics if the underlying [`sysctlbyname`] call fails.
pub fn total() -> u64 {
    try_get_total_physical_memory().expect("sysctlbyname")
}

/// Get the memory statistics about the host mac.
pub fn vm_statistics() -> Result<vm_statistics64, Errno> {
    let mach_port = unsafe { mach_host_self() };
    let mut count = HOST_VM_INFO64_COUNT;

    // SAFETY: vm_statistics64 does not contain pointers/refs that would be null, so this is safe/valid.
    let mut stats: vm_statistics64 = unsafe { mem::zeroed() };

    let return_value = unsafe {
        host_statistics64(
            mach_port,
            HOST_VM_INFO64,
            &mut stats as *mut _ as host_info64_t,
            &mut count,
        )
    };

    // Do this because heim does this and I assume they know better than me.
    let port_result = unsafe { mach_port_deallocate(mach_task_self(), mach_port) };

    if port_result != KERN_SUCCESS || return_value != KERN_SUCCESS {
        Err(errno::errno())
    } else {
        Ok(stats)
    }
}

/// Calculate the avialable system memory using [`vm_statistics`].
///
/// # Panics
/// - Panics if [`vm_statistics`] errors.
/// - Panics if [`page_size`] errors.
pub fn calculate_available_memory() -> u64 {
    let page_size = page_size().expect("error getting page size");
    let stats = vm_statistics().expect("error getting vm_statistics64");

    // This is how heim calculates it so we will too -- I wish mac os had better docs for this.
    (stats.active_count + stats.free_count) as u64 * page_size
}
