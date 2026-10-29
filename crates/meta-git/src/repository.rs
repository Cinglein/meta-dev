use std::fmt;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use crate::change::Change;
use crate::git_failure::GitFailure;
use crate::layout::Layout;
use crate::query::Query;
use crate::reads::Reads;
use crate::writes::Writes;

#[derive(Clone)]
pub struct Repository {
    path: PathBuf,
    opened: Arc<Mutex<git2::Repository>>,
}

impl Repository {
    const REF_STORAGE: &'static str = "extensions.refstorage";
    const REFTABLE: &'static str = "reftable";

    pub fn open(path: &Path) -> Result<Repository, GitFailure> {
        let reftable = || GitFailure::Reftable(path.to_path_buf());
        let opened = git2::Repository::open(path).map_err(|error| {
            if error.message().contains(Self::REF_STORAGE) { reftable() } else { GitFailure::from(error) }
        })?;
        let storage = opened.config()?.get_string(Self::REF_STORAGE).ok();
        match storage.as_deref() {
            Some(Self::REFTABLE) => Err(reftable()),
            _ => Ok(Repository { path: path.to_path_buf(), opened: Arc::new(Mutex::new(opened)) }),
        }
    }

    pub fn created(path: &Path, layout: Layout) -> Result<Repository, GitFailure> {
        git2::Repository::init_opts(path, &layout.options())?;
        Self::open(path)
    }

    fn locked(&self) -> MutexGuard<'_, git2::Repository> {
        self.opened.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

impl Reads for Repository {
    fn read<Asked: Query>(&self, query: &Asked) -> Result<Asked::View, GitFailure> {
        query.answered(&self.locked())
    }
}

impl Writes for Repository {
    fn write<Made: Change>(&self, change: &Made) -> Result<Made::Applied, GitFailure> {
        change.applied(&self.locked())
    }
}

impl fmt::Debug for Repository {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.debug_tuple("Repository").field(&self.path).finish()
    }
}
