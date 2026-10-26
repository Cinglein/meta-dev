use std::fmt;

use super::cause::Cause;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Kept {
    NotFound,
    Symlink,
    Escaped,
    Unlinked,
    Locked,
    Borrowed,
    NestedRepo,
    Unreadable,
    IndexFlags,
    Submodules,
    Uncommitted,
    Untracked,
    Ignored,
    Diverged,
    Unpushed,
    Unidentified,
    Stranger,
    OffBranch,
    NoBranch,
    Shared,
    Paused,
    Redirected,
    Unreachable(Cause),
    InUse,
    Changed,
    Uninspected(Cause),
    Unremoved(Cause),
}

impl Kept {
    pub fn recast(self, recast: fn(Cause) -> Self) -> Self {
        match self {
            Self::Uninspected(cause) => recast(cause),
            other => other,
        }
    }
}

impl fmt::Display for Kept {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotFound => formatter.write_str("not found"),
            Self::Symlink => formatter.write_str("symlink"),
            Self::Escaped => formatter.write_str("path escapes the worktree root"),
            Self::Unlinked => formatter.write_str("not a linked worktree of this clone"),
            Self::Locked => formatter.write_str("locked"),
            Self::Borrowed => formatter.write_str("borrowed objects"),
            Self::NestedRepo => formatter.write_str("nested repo"),
            Self::Unreadable => formatter.write_str("unreadable entry"),
            Self::IndexFlags => formatter.write_str("index entry not plain"),
            Self::Submodules => formatter.write_str("has submodules"),
            Self::Uncommitted => formatter.write_str("uncommitted changes"),
            Self::Untracked => formatter.write_str("untracked files"),
            Self::Ignored => formatter.write_str("ignored files besides cargo target dirs"),
            Self::Diverged => formatter.write_str("branch differs from the remote"),
            Self::Unpushed => formatter.write_str("commits not on the remote"),
            Self::Unidentified => formatter.write_str("no recorded worktree id"),
            Self::Stranger => formatter.write_str("a different worktree at the recorded path"),
            Self::OffBranch => formatter.write_str("HEAD is not on the recorded branch"),
            Self::NoBranch => formatter.write_str("task branch missing"),
            Self::Shared => formatter.write_str("branch checked out elsewhere"),
            Self::Paused => formatter.write_str("am, rebase or cherry-pick in progress"),
            Self::Redirected => formatter.write_str("worktree config sets a remote"),
            Self::Unreachable(cause) => write!(formatter, "remote unreachable: {cause}"),
            Self::InUse => formatter.write_str("in use"),
            Self::Changed => formatter.write_str("changed during checks"),
            Self::Uninspected(cause) => write!(formatter, "cannot inspect: {cause}"),
            Self::Unremoved(cause) => write!(formatter, "removal failed: {cause}"),
        }
    }
}
