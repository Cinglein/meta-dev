use std::fmt;

use crate::request::cli_failure::Message;

#[derive(Debug)]
pub enum Kept {
    NotFound,
    Symlink,
    HoldsCurrentRepo,
    Unrecognised,
    Uncommitted,
    Untracked,
    Ignored,
    Unpushed,
    LinkedWorktrees,
    InUse,
    Uninspected(Message),
    Unremoved(Message),
}

impl fmt::Display for Kept {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotFound => formatter.write_str("not found"),
            Self::Symlink => formatter.write_str("symlink"),
            Self::HoldsCurrentRepo => formatter.write_str("holds the current repo"),
            Self::Unrecognised => formatter.write_str("neither a cargo target dir nor a git work tree root"),
            Self::Uncommitted => formatter.write_str("uncommitted changes"),
            Self::Untracked => formatter.write_str("untracked files"),
            Self::Ignored => formatter.write_str("ignored files besides cargo target dirs"),
            Self::Unpushed => formatter.write_str("commits not on a remote"),
            Self::LinkedWorktrees => formatter.write_str("has linked worktrees"),
            Self::InUse => formatter.write_str("in use"),
            Self::Uninspected(cause) => write!(formatter, "cannot inspect: {cause}"),
            Self::Unremoved(cause) => write!(formatter, "removal failed: {cause}"),
        }
    }
}
