use std::fs::File;

use meta_door::{MetaStore, Opening};

use crate::request::cli_failure::{CliFailure, Setup};
use crate::request::local::LocalFile;

#[derive(Debug)]
pub struct GcLock {
    _held: File,
}

impl GcLock {
    const FILE: &'static str = "meta-local/gc.lock";
    const HELD: &'static str = "another meta gc is running in this clone";

    pub fn taken(store: &MetaStore) -> Result<Self, CliFailure> {
        let file = LocalFile::new(store.entry(Self::FILE)).opened(Opening::Held)?;
        file.try_lock().map_err(|_| CliFailure::raised::<Setup>(Self::HELD))?;
        Ok(Self { _held: file })
    }
}

#[cfg(test)]
mod tests {
    use super::GcLock;
    use crate::request::tests::Repository;

    #[test]
    fn a_second_gc_in_the_same_clone_is_refused_while_the_first_holds_the_lock() {
        let repository = Repository::new();
        let first = GcLock::taken(repository.store());
        assert!(first.is_ok() && GcLock::taken(repository.store()).is_err());
    }
}
