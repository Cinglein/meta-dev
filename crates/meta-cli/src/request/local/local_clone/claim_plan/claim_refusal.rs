use std::fmt;
use std::path::PathBuf;

use meta_log::RemoteName;

use crate::request::project::BranchName;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ClaimRefusal {
    Unheld(BranchName),
    HeadUnset(RemoteName),
    Taken(PathBuf),
    NoBase(BranchName),
}

impl fmt::Display for ClaimRefusal {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unheld(branch) => write!(formatter, "branch {branch} exists but you do not hold the task"),
            Self::HeadUnset(remote) => {
                write!(formatter, "{remote}/HEAD is unset; run git remote set-head {remote} --auto")
            }
            Self::Taken(path) => write!(formatter, "{} exists; move it away first", path.display()),
            Self::NoBase(base) => write!(formatter, "{base}: no such commit"),
        }
    }
}
