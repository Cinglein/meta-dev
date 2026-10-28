use std::path::Path;

use meta_door::{Change, Door, MetaStore};
use tempfile::TempDir;

pub struct Repository {
    directory: TempDir,
    store: MetaStore,
}

impl Repository {
    const GIT: &'static str = ".git";

    pub fn new() -> Self {
        let directory = tempfile::tempdir().unwrap();
        Door::changed(directory.path().join(Self::GIT), Change::Dir).unwrap();
        let store = MetaStore::located(directory.path()).unwrap();
        Self { directory, store }
    }

    pub fn path(&self) -> &Path {
        self.directory.path()
    }

    pub fn store(&self) -> &MetaStore {
        &self.store
    }
}
