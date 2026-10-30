use std::fmt;
use std::num::NonZeroU32;

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct SchemaVersion {
    version: NonZeroU32,
}

impl SchemaVersion {
    const FIRST_KEEPING_HOLDERS: Self = Self { version: NonZeroU32::MIN.saturating_add(6) };
    const PROJECTS: Self = Self::FIRST_KEEPING_HOLDERS.next();

    const fn next(self) -> Self {
        Self { version: self.version.saturating_add(NonZeroU32::MIN.get()) }
    }

    #[must_use]
    pub fn keeps_holders_on_reopen(self) -> bool {
        self >= Self::FIRST_KEEPING_HOLDERS
    }

    #[must_use]
    pub fn current() -> Self {
        Self::PROJECTS
    }
}

impl fmt::Display for SchemaVersion {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.version.fmt(formatter)
    }
}
