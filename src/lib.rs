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

/// Snapshot of the host's memory stats.
#[derive(Debug, Clone, Copy)]
pub struct Snapshot {
    /// Total number of bytes of physical memory on the host system.
    pub total: u64,

    /// Number of bytes of available physical memory on the host system.
    pub available: u64,
}

impl Snapshot {
    /// Try to get a snapshot of the state of the system's memory
    #[cfg(any(windows, target_os = "linux", target_os = "macos", target_os = "ios"))]
    #[allow(unreachable_code)]
    pub fn get() -> Result<Self, Option<errno::Errno>> {
        #[cfg(windows)] {
            let mem_status = windows::populate_mem_status()?;

            return Ok(Self {
                total: mem_status.ullTotalPhys,
                available: mem_status.ullAvailPhys,
            });
        }

        // sysinfo.totalram is a C unsigned long, which is a u32 on some targets.
        // cast to u64 just to be sure.
        #[cfg(target_os = "linux")] {
            let sysinfo = linux::populate_sysinfo()?;

            return Ok(Self {
                total: sysinfo.totalram,
                available: sysinfo.freeram,
            });
        }

        #[cfg(any(target_os = "macos", target_os = "ios"))] {
            let total_memory = macos::try_get_total_physical_memory()?;
            let page_size = macos::page_size()?;
            let vm_stats = macos::vm_statistics()
                .map_err(|errno| if errno.0 == 0 { None } else { Some(errno) })?;

            return Ok(Self {
                total: total_memory,
                // This is how heim calculates it so we will too -- I wish macOS
                // had better docs for this.
                available: (vm_stats.active_count + vm_stats.free_count) as u64 * page_size,
            });
        }

        unreachable!("This function should have already hit a CFG and returned");
    }

    /// Get the number of bytes of memory in use at the time of this snapshot.
    pub fn in_use(&self) -> u64 {
        self.total - self.available
    }
}


#[cfg(any(windows, target_os = "linux", target_os = "macos", target_os = "ios"))]
#[inline]
fn get_snapshot() -> Snapshot {
    Snapshot::get().expect("failed to query system for memory stats")
}

/// Get the total number of bytes of physical memory on this host.
///
/// # Panics
/// This function may panic if any of the underlying platform-specific syscalls fail.
#[cfg(any(windows, target_os = "linux", target_os = "macos", target_os = "ios"))]
pub fn total() -> u64 {
    get_snapshot().total
}

/// Get the number of bytes of available physical memory on this host.
///
/// # Panics
/// This function may panic if any of the underlying platform-specific syscalls fail.
#[cfg(any(windows, target_os = "linux", target_os = "macos", target_os = "ios"))]
pub fn available() -> u64 {
    get_snapshot().available
}

/// Get the number of bytes of physical memory currently in use.
///
/// # Panics
/// This function may panic if any of the underlying platform-specific syscalls fail.
#[cfg(any(windows, target_os = "linux", target_os = "macos", target_os = "ios"))]
pub fn used() -> u64 {
    get_snapshot().in_use()
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
