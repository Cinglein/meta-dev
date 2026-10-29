use std::path::PathBuf;

use super::git_call::GitCall;

// A read. It changes nothing, so the interpreter may run the reads of one Look side by side.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Look {
    Entry(PathBuf),
    Canonical(PathBuf),
    Text(PathBuf),
    Git(GitCall),
    Walk(PathBuf),
    Lenders(PathBuf),
    Occupancy(Vec<PathBuf>),
}
