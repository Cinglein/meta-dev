use std::fmt;

use serde::{Deserialize, Serialize};

use crate::byte_count::ByteCount;
use crate::removed_kind::RemovedKind;
use crate::text::Text;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Removal {
    removed: RemovedKind,
    bytes: ByteCount,
    place: Option<Text>,
}

impl Removal {
    #[must_use]
    pub fn new(removed: RemovedKind, bytes: ByteCount, place: Option<Text>) -> Self {
        Self { removed, bytes, place }
    }
}

impl fmt::Display for Removal {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{} {}", self.removed, self.bytes)?;
        self.place.iter().try_for_each(|place| write!(formatter, " {place}"))
    }
}
