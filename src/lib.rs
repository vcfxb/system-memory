//! A small crate that resolves the total system memory of the host. This is useful for many projects that 
//! may behave differently depending on how much memory the host system has and how much is available.

#![deny(missing_copy_implementations, missing_debug_implementations)]
#![deny(rustdoc::broken_intra_doc_links)]
#![warn(missing_docs)]
#![no_std]

// Compiler directive to get docs.rs (which uses the nightly version of the rust compiler) to show
// info about feature required for various modules and functionality.
//
// See: <https://stackoverflow.com/a/70914430>.
#![cfg_attr(all(doc, CHANNEL_NIGHTLY), feature(doc_auto_cfg))]

#[cfg(windows)]
pub mod windows;

/// Get the number of bytes of phiscal memory on this host.
/// 
/// # Panics
/// This function may panic if any of the underlying platform-specific syscalls fail.
#[cfg(any(windows, unix))]
#[allow(unreachable_code)]
pub fn total() -> u64 {
    #[cfg(windows)]
    return windows::total_physical_memory();

    unreachable!("This function should have already hit a CFG and returned");
}


#[cfg(test)]
mod tests {
    extern crate std;
    use std::println;

    #[test]
    fn get_total_system_memory() {
        println!("Total system memory: {:.2} GiB", super::total() as f64 / 1024f64 / 1024f64 / 1024f64);
    }
}
