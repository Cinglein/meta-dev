use std::fmt;
use std::num::NonZeroU32;

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct SchemaVersion {
    version: NonZeroU32,
}

impl SchemaVersion {
    const FIRST_KEEPING_HOLDERS: Self = Self { version: NonZeroU32::MIN.saturating_add(6) };

    #[must_use]
    pub fn keeps_holders_on_reopen(self) -> bool {
        self >= Self::FIRST_KEEPING_HOLDERS
    }

    #[must_use]
    pub fn current() -> Self {
        Self {
            version: NonZeroU32::MIN
                .saturating_add(1)
                .saturating_add(1)
                .saturating_add(1)
                .saturating_add(1)
                .saturating_add(1)
                .saturating_add(1),
        }
    }
}

impl fmt::Display for SchemaVersion {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.version.fmt(formatter)
    }
}
