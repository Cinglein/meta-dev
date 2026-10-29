use std::path::PathBuf;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub(super) struct Span {
    file_name: PathBuf,
    is_primary: bool,
}

impl Span {
    pub(super) fn is_primary(&self) -> bool {
        self.is_primary
    }

    pub(super) fn path(&self) -> PathBuf {
        self.file_name.clone()
    }
}
