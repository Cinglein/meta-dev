#[cfg(target_os = "macos")]
mod darwin;
mod held;
#[cfg(any(target_os = "linux", all(test, unix)))]
mod procfs;
mod scan;
#[cfg(not(any(target_os = "macos", target_os = "linux")))]
mod unsupported;

#[cfg(test)]
mod tests;

use crate::request::local::local_clone::vetting::cause::Cause;
#[cfg(target_os = "macos")]
use darwin::Darwin as Platform;
pub use held::Held;
#[cfg(target_os = "linux")]
use procfs::Procfs as Platform;
#[cfg(test)]
pub use scan::Scan;
#[cfg(not(any(target_os = "macos", target_os = "linux")))]
use unsupported::Unsupported as Platform;

#[derive(Debug)]
pub struct ProcessTable;

impl ProcessTable {
    pub fn held() -> Result<Held, Cause> {
        Platform::held()
    }
}
