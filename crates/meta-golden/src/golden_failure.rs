use std::io;
use std::path::PathBuf;

use meta_run::Unread;
use thiserror::Error;

use crate::argv::Argv;
use crate::difference::Difference;

#[derive(Debug, Error)]
pub enum GoldenFailure {
    #[error("{}: {source}", path.display())]
    Unreadable { path: PathBuf, source: io::Error },
    #[error("{}: not a recording: {reason}", path.display())]
    Malformed { path: PathBuf, reason: String },
    #[error("{}: {difference}; run `cargo xtask goldens` and review the diff", path.display())]
    Differs { path: PathBuf, difference: Difference },
    #[error("{}: a setup step failed: {unread}", scenario.display())]
    Unbuilt { scenario: PathBuf, unread: Unread },
    #[error("no recording answers {0}")]
    Unanswered(Argv),
}
