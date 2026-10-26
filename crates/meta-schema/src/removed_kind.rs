use std::fmt;

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RemovedKind {
    TargetDir,
    Worktree,
    Clone,
}

impl fmt::Display for RemovedKind {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::TargetDir => "target-dir",
            Self::Worktree => "worktree",
            Self::Clone => "clone",
        })
    }
}
