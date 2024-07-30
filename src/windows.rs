//! Windows specific implementation of getting memory info/status.

extern crate alloc;

use windows_sys::{core::PSTR, Win32::{
    Foundation::{GetLastError, LocalFree},
    System::{
        Diagnostics::Debug::{FormatMessageA, FORMAT_MESSAGE_ALLOCATE_BUFFER, FORMAT_MESSAGE_FROM_SYSTEM, FORMAT_MESSAGE_IGNORE_INSERTS, FORMAT_MESSAGE_OPTIONS}, 
        SystemInformation::{GlobalMemoryStatusEx, MEMORYSTATUSEX}
    }
}};

use core::{ffi::{c_void, CStr}, mem, ptr};

use alloc::string::String;

/// Appropriately populate a [MEMORYSTATUSEX] object using the [GlobalMemoryStatusEx] syscall.
/// 
/// If [GlobalMemoryStatusEx] errors, return the error code from [GlobalMemoryStatusEx].
pub fn populate_mem_status() -> Result<MEMORYSTATUSEX, i32> {
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
        Err(return_value)
    }
}

/// Calls [populate_mem_status] and then additionally converts any errors that are encountered to an ASCII 
/// error message.
/// 
/// # Panics
/// - This will panic if windows ([`FormatMessageA`]) returns an error message that is not valid UTF-8.
pub fn populate_mem_status_or_err_msg() -> Result<MEMORYSTATUSEX, String> {
    match populate_mem_status() {
        Ok(mem_status) => Ok(mem_status),

        Err(_) => {
            // Call into the windows API to get the error code.
            let error_code = unsafe { GetLastError() };
                
            // Null pointer for the message buffer that FormatMessageA will populate.
            let mut message_buffer: PSTR = ptr::null_mut();

            // Build the flags to pass to FormatMessageA.
            let flags: FORMAT_MESSAGE_OPTIONS = 0 
                // Allocate the buffer with the message.
                | FORMAT_MESSAGE_ALLOCATE_BUFFER 
                // Get the message from the system
                | FORMAT_MESSAGE_FROM_SYSTEM 
                // Ignore formatting characters -- there shouldn't be any to my knowledge.
                | FORMAT_MESSAGE_IGNORE_INSERTS;

            // Format the message into the buffer and get the number of ascii chars formatted.
            let _ascii_chars = unsafe { 
                FormatMessageA(
                    flags,
                    ptr::null(),
                    error_code,
                    // Let FormatMessageA determine a language on its own according to 
                    // https://learn.microsoft.com/en-us/windows/win32/api/winbase/nf-winbase-formatmessagea. 
                    0,
                    // Despite this actually being a pointer-to-a-pointer, FormatMessageA requires us to cast it 
                    // to a byte pointer (PSTR) before passing it in.
                    (&mut message_buffer as *mut PSTR) as PSTR,  
                    0, 
                    ptr::null()
                ) 
            };

            // SAFETY: Assume that FormatMessageA properly wrote to the buffer.
            let c_str: &CStr = unsafe { CStr::from_ptr(message_buffer as *const _) };
            
            // Convert to an &str.
            let err_str: &str = c_str
                .to_str()
                .expect("ASCII error message from windows is valid UTF-8");

            // Make sure we own the string so we can free the buffer that windows allocated.
            let owned: String = String::from(err_str);

            // Free the buffer we gave windows.
            unsafe { LocalFree(message_buffer as *mut c_void) };

            Err(owned)
        }
    }
}


/// Populate a [MEMORYSTATUSEX] properly or panic.
/// 
/// # Panics
/// - If the underlying system call to windows returns an error.
/// - If the underlying system call to windows returns an error message that is not valid UTF-8.
pub fn mem_status() -> MEMORYSTATUSEX {
    populate_mem_status_or_err_msg().expect("could not get system memory status")
}

/// Get the number of physical bytes of memory on the system's hardware.
/// 
/// # Panics
/// - If the underlying system call to windows returns an error.
/// - If the underlying system call to windows returns an error message that is not valid UTF-8.
pub fn total_physical_memory() -> u64 {
    mem_status().ullTotalPhys
}

/// Get the number of available physical bytes of memory on the system's hardware.
/// 
/// # Panics
/// - If the underlying system call to windows returns an error.
/// - If the underlying system call to windows returns an error message that is not valid UTF-8.
pub fn available_physical_memory() -> u64 {
    mem_status().ullAvailPhys
}
