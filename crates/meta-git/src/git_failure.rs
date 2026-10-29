use std::path::PathBuf;

use meta_run::RunFailure;
use thiserror::Error;

use crate::ref_name::RefName;

#[derive(Debug, Error)]
pub enum GitFailure {
    #[error("{0} keeps refs in reftable; meta reads only files refs; nothing done")]
    Reftable(PathBuf),
    #[error("git: {0}")]
    Library(#[from] git2::Error),
    #[error("offline: {0}")]
    Unrun(#[from] RunFailure),
    #[error("{0} is not UTF-8 text")]
    NotText(String),
    #[error("not an object id: {0:?}")]
    NotObjectId(String),
    #[error("{0} moved since it was read; nothing done")]
    RefMoved(RefName),
}

impl GitFailure {
    pub fn ref_moved(self) -> Result<RefName, GitFailure> {
        match self {
            Self::RefMoved(name) => Ok(name),
            other => Err(other),
        }
    }
}
