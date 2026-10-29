#[cfg(target_os = "macos")]
mod darwin;
mod held;
#[cfg(any(target_os = "linux", all(test, unix)))]
mod procfs;
mod scan;
mod table_failure;
mod uninspected;
#[cfg(not(any(target_os = "macos", target_os = "linux")))]
mod unsupported;

#[cfg(test)]
mod tests;

#[cfg(target_os = "macos")]
use darwin::Darwin as Platform;
pub use held::Held;
#[cfg(target_os = "linux")]
use procfs::Procfs as Platform;
pub use scan::Scan;
pub use table_failure::TableFailure;
pub use uninspected::Uninspected;
#[cfg(not(any(target_os = "macos", target_os = "linux")))]
use unsupported::Unsupported as Platform;

#[derive(Debug)]
pub struct ProcessTable;

impl ProcessTable {
    pub fn held() -> Result<Held, TableFailure> {
        Platform::held()
    }
}
