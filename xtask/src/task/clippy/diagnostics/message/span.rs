mod role;

use std::path::PathBuf;

use serde::Deserialize;

use role::Role;

#[derive(Debug, Deserialize)]
pub(super) struct Span {
    file_name: PathBuf,
    is_primary: Role,
}

impl Span {
    pub(super) fn is_primary(&self) -> bool {
        self.is_primary == Role::Primary
    }

    pub(super) fn path(&self) -> PathBuf {
        self.file_name.clone()
    }
}
