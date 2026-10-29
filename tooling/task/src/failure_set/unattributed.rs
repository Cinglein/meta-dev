use std::io;
use std::path::PathBuf;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum Unattributed {
    #[error("unattributed: no failures file at {}", .0.display())]
    Missing(PathBuf),
    #[error("unattributed: {} is unreadable: {error}", path.display())]
    Unreadable { path: PathBuf, error: io::Error },
    #[error("unattributed: failures are not the shape expected: {0}")]
    Malformed(serde_json::Error),
}
