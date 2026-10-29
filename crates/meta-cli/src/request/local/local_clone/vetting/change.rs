use std::path::PathBuf;

use super::git_call::GitCall;
use super::thaw::Thaw;

// A write. Changes run one at a time, in order. `Destroy` is a write that can lose work:
// the interpreter runs it only straight after a Look, so it never acts on an old view.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Change {
    Git(GitCall),
    Destroy(GitCall),
    Freeze(PathBuf, Thaw),
    Thaw(Thaw),
}

impl Change {
    pub fn destroys(&self) -> bool {
        matches!(self, Self::Destroy(_))
    }
}
