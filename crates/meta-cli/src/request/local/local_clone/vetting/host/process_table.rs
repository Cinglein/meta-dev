#[cfg(target_os = "macos")]
mod darwin;
#[cfg(target_os = "linux")]
mod procfs;
#[cfg(not(any(target_os = "macos", target_os = "linux")))]
mod unsupported;

#[cfg(test)]
mod tests;

use std::path::PathBuf;

use crate::request::local::local_clone::vetting::occupancy::Occupancy;
#[cfg(target_os = "macos")]
use darwin::Darwin as Platform;
#[cfg(target_os = "linux")]
use procfs::Procfs as Platform;
#[cfg(not(any(target_os = "macos", target_os = "linux")))]
use unsupported::Unsupported as Platform;

#[derive(Debug)]
pub struct ProcessTable;

impl ProcessTable {
    pub fn occupancy(scanned: &[PathBuf]) -> Occupancy {
        Platform::held().map_or_else(Occupancy::Unlisted, |held| Occupancy::Listed(Self::under(held, scanned)))
    }

    fn under(held: Vec<PathBuf>, scanned: &[PathBuf]) -> Vec<PathBuf> {
        held.into_iter().filter(|path| scanned.iter().any(|root| path.starts_with(root))).collect()
    }
}
