//! A small crate that resolves the total system memory of the host. This is useful for many projects that
//! may behave differently depending on how much memory the host system has and how much is available.
//!
//! Be aware of the potential for data races when using this code -- since the amount of available system memory may (
//! and likely will) change between calls, repeated use of the function even on the same thread cannot be expected to
//! return the same values, nor will [`available`] necessarily return values consistent with [`used`], since the value
//! may change between calls.

#![deny(missing_copy_implementations, missing_debug_implementations)]
#![deny(rustdoc::broken_intra_doc_links)]
#![deny(clippy::cast_possible_truncation)]
#![warn(missing_docs)]
#![no_std]
// Compiler directive to get docs.rs (which uses the nightly version of the rust compiler) to show
// info about feature required for various modules and functionality.
//
// See: <https://stackoverflow.com/a/70914430>.
#![cfg_attr(all(doc, CHANNEL_NIGHTLY), feature(doc_auto_cfg))]

#[cfg(windows)]
pub mod windows;

#[cfg(any(target_os = "macos", target_os = "ios"))]
pub mod macos;

#[cfg(target_os = "linux")]
pub mod linux;

/// Get the total number of bytes of physical memory on this host.
///
/// # Panics
/// This function may panic if any of the underlying platform-specific syscalls fail.
#[cfg(any(windows, target_os = "linux", target_os = "macos", target_os = "ios"))]
#[allow(unreachable_code)]
pub fn total() -> u64 {
    #[cfg(windows)]
    return windows::mem_status().ullTotalPhys;

    // sysinfo.totalram is a C unsigned long, which is a u32 on some targets.
    // cast to u64 just to be sure.
    #[cfg(target_os = "linux")]
    return linux::get_sysinfo().totalram as u64;

    #[cfg(any(target_os = "macos", target_os = "ios"))]
    return macos::total();

    unreachable!("This function should have already hit a CFG and returned");
}

/// Get the number of bytes of available physical memory on this host.
///
/// # Panics
/// This function may panic if any of the underlying platform-specific syscalls fail.
#[cfg(any(windows, target_os = "linux", target_os = "macos", target_os = "ios"))]
#[allow(unreachable_code)]
pub fn available() -> u64 {
    #[cfg(windows)]
    return windows::mem_status().ullAvailPhys;

    // sysinfo.freeram is the same as sysinfo.totalram above.
    #[cfg(target_os = "linux")]
    return linux::get_sysinfo().freeram as u64;

    #[cfg(any(target_os = "macos", target_os = "ios"))]
    return macos::calculate_available_memory();

    unreachable!("This function should have already hit a CFG and returned");
}

/// Get the number of bytes of physical memory currently in use.
///
/// # Panics
/// This function may panic if any of the underlying platform-specific syscalls fail.
#[cfg(any(windows, target_os = "linux", target_os = "macos", target_os = "ios"))]
pub fn used() -> u64 {
    total() - available()
}

#[cfg(test)]
mod tests {
    extern crate std;
    use std::println;

    #[test]
    fn get_total_system_memory() {
        println!(
            "Total system memory: {:.2} GiB",
            super::total() as f64 / 1024f64 / 1024f64 / 1024f64
        );
        println!(
            "Available system memory: {:.2} GiB",
            super::available() as f64 / 1024f64 / 1024f64 / 1024f64
        );

        // This may not always assert successfully -- there's a race condition here if the amount of available memory
        // changes after the call to `super::used`.
        assert_eq!(super::used(), super::total() - super::available());
    }
}
