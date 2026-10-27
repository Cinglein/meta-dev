use std::ffi::OsStr;
use std::path::{Component, Path, PathBuf};

use super::entry_kind::EntryKind;
use super::shared_node::SharedNode;
use crate::request::local::byte_count::ByteCount;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Found {
    path: PathBuf,
    kind: EntryKind,
    bytes: ByteCount,
    shared: Option<SharedNode>,
}

impl Found {
    const ALTERNATES: [&'static str; 3] = ["alternates", "info", "objects"];

    pub fn new(path: PathBuf, kind: EntryKind, bytes: ByteCount) -> Self {
        Self { path, kind, bytes, shared: None }
    }

    pub fn sharing(self, node: SharedNode) -> Self {
        Self { shared: Some(node), ..self }
    }

    pub fn shared(&self) -> Option<SharedNode> {
        self.shared
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

    pub fn lends(&self) -> bool {
        let tail: Vec<&OsStr> =
            self.path.components().rev().take(Self::ALTERNATES.len()).map(Component::as_os_str).collect();
        tail.len() == Self::ALTERNATES.len()
            && tail.iter().zip(Self::ALTERNATES).all(|(part, name)| part.eq_ignore_ascii_case(name))
    }
}
