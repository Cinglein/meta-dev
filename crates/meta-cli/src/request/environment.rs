use std::collections::BTreeMap;
use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, PoisonError};

use super::follow_up::FollowUp;

#[derive(Clone, Debug, Default)]
pub struct Environment {
    directory: PathBuf,
    program: PathBuf,
    variables: BTreeMap<OsString, OsString>,
    then: Arc<Mutex<Vec<FollowUp>>>,
}

impl Environment {
    const TEMPORARY: &'static str = "TMPDIR";
    const HOME: &'static str = "HOME";
    const SHARED_TEMPORARY: &'static str = "/tmp";

    #[must_use]
    pub fn new(directory: PathBuf, program: PathBuf, variables: BTreeMap<OsString, OsString>) -> Self {
        Self { directory, program, variables, then: Arc::default() }
    }

    #[must_use]
    pub fn directory(&self) -> &Path {
        &self.directory
    }

    #[must_use]
    pub fn program(&self) -> &OsStr {
        self.program.as_os_str()
    }

    #[must_use]
    pub fn variable(&self, name: &str) -> Option<&OsStr> {
        self.variables.get(OsStr::new(name)).map(OsString::as_os_str)
    }

    #[must_use]
    pub fn text(&self, name: &str) -> Option<String> {
        self.variable(name).and_then(OsStr::to_str).map(str::to_owned)
    }

    #[must_use]
    pub fn temporary(&self) -> PathBuf {
        PathBuf::from(self.variable(Self::TEMPORARY).unwrap_or(OsStr::new(Self::SHARED_TEMPORARY)))
    }

    #[must_use]
    pub fn home(&self) -> Option<PathBuf> {
        self.variable(Self::HOME).map(PathBuf::from)
    }

    #[must_use]
    pub fn moved_to(&self, directory: PathBuf) -> Self {
        Self { directory, ..self.clone() }
    }

    pub fn then(&self, follow_up: FollowUp) {
        self.then.lock().unwrap_or_else(PoisonError::into_inner).push(follow_up);
    }

    pub(crate) fn follow_ups(&self) -> Vec<FollowUp> {
        std::mem::take(&mut *self.then.lock().unwrap_or_else(PoisonError::into_inner))
    }
}
