use std::fmt;

use serde::{Deserialize, Serialize};

use crate::byte_count::ByteCount;
use crate::removed_kind::RemovedKind;
use crate::text::Text;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Removal {
    path: Text,
    removed: RemovedKind,
    bytes: ByteCount,
}

impl Removal {
    #[must_use]
    pub fn new(path: Text, removed: RemovedKind, bytes: ByteCount) -> Self {
        Self { path, removed, bytes }
    }
}

impl fmt::Display for Removal {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{} {} {}", self.removed, self.bytes, self.path)
    }
}
