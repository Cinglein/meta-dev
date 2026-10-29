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

    pub fn open(path: &Path) -> Result<Repository, GitFailure> {
        let opened = git2::Repository::open(path).map_err(|error| {
            if error.message().contains(Self::REF_STORAGE) {
                GitFailure::Reftable(path.to_path_buf())
            } else {
                GitFailure::from(error)
            }
        })?;
        Ok(Repository { path: path.to_path_buf(), opened: Arc::new(Mutex::new(opened)) })
    }

    pub fn discovered(path: &Path) -> Result<Option<Repository>, GitFailure> {
        match git2::Repository::discover_path(path, std::iter::empty::<&Path>()) {
            Ok(found) => Self::open(&found).map(Some),
            Err(error) if error.code() == git2::ErrorCode::NotFound => Ok(None),
            Err(error) => Err(GitFailure::from(error)),
        }
    }

    pub fn created(path: &Path, layout: Layout) -> Result<Repository, GitFailure> {
        git2::Repository::init_opts(path, &layout.options())?;
        Self::open(path)
    }

    #[must_use]
    pub fn git_dir(&self) -> PathBuf {
        self.locked().path().to_path_buf()
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
