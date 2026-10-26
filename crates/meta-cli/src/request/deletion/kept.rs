use std::fmt;

use crate::request::cli_failure::Message;

#[derive(Debug)]
pub enum Kept {
    NotFound,
    Symlink,
    HoldsCurrentRepo,
    Unrecognised,
    HoldsRepo,
    Foreign,
    IndexFlags,
    Submodules,
    Uncommitted,
    Untracked,
    Ignored,
    Unpushed,
    LinkedWorktrees,
    Borrowed,
    InUse,
    Changed,
    Uninspected(Message),
    Unremoved(Message),
}

impl Kept {
    pub fn nested(self) -> Self {
        match self {
            Self::Unrecognised => Self::Ignored,
            other => other,
        }
    }
}

impl fmt::Display for Kept {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotFound => formatter.write_str("not found"),
            Self::Symlink => formatter.write_str("symlink"),
            Self::HoldsCurrentRepo => formatter.write_str("holds the current repo"),
            Self::Unrecognised => formatter.write_str("neither a cargo target dir nor a git work tree root"),
            Self::HoldsRepo => formatter.write_str("cache tag over a git repo"),
            Self::Foreign => formatter.write_str("not cargo's layout"),
            Self::IndexFlags => formatter.write_str("index flags set"),
            Self::Submodules => formatter.write_str("has submodules"),
            Self::Uncommitted => formatter.write_str("uncommitted changes"),
            Self::Untracked => formatter.write_str("untracked files"),
            Self::Ignored => formatter.write_str("ignored files besides cargo target dirs"),
            Self::Unpushed => formatter.write_str("commits not on a remote"),
            Self::LinkedWorktrees => formatter.write_str("has linked worktrees"),
            Self::Borrowed => formatter.write_str("objects borrowed by another repo"),
            Self::InUse => formatter.write_str("in use"),
            Self::Changed => formatter.write_str("changed during checks"),
            Self::Uninspected(cause) => write!(formatter, "cannot inspect: {cause}"),
            Self::Unremoved(cause) => write!(formatter, "removal failed: {cause}"),
        }
    }
}
