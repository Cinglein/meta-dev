use std::fs::File;
use std::path::Path;

use crate::request::cli_failure::{CliFailure, Setup};

#[derive(Debug)]
pub struct GcLock {
    _held: File,
}

impl GcLock {
    const FILE: &'static str = "meta-local/gc.lock";
    const HELD: &'static str = "another meta gc is running in this clone";

    pub fn taken(common: &Path) -> Result<Self, CliFailure> {
        let path = common.join(Self::FILE);
        path.parent().map_or(Ok(()), std::fs::create_dir_all).map_err(CliFailure::raised::<Setup>)?;
        let file = File::options()
            .create(true)
            .truncate(false)
            .write(true)
            .open(&path)
            .map_err(CliFailure::raised::<Setup>)?;
        file.try_lock().map_err(|_| CliFailure::raised::<Setup>(Self::HELD))?;
        Ok(Self { _held: file })
    }
}

#[cfg(test)]
mod tests {
    use super::GcLock;

    #[test]
    fn one_gc_holds_the_lock_until_it_ends() {
        let common = tempfile::tempdir().unwrap();
        let first = GcLock::taken(common.path());
        let second = GcLock::taken(common.path());
        drop(first);
        assert!(second.is_err() && GcLock::taken(common.path()).is_ok());
    }
}
