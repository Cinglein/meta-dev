use std::path::{Path, PathBuf};

use super::entry_kind::EntryKind;
use crate::request::local::byte_count::ByteCount;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Found {
    path: PathBuf,
    kind: EntryKind,
    bytes: ByteCount,
}

impl Found {
    pub fn new(path: PathBuf, kind: EntryKind, bytes: ByteCount) -> Self {
        Self { path, kind, bytes }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn kind(&self) -> EntryKind {
        self.kind
    }

    pub fn bytes(&self) -> ByteCount {
        self.bytes
    }
}
