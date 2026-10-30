use std::fmt;
use std::num::NonZeroU32;

use serde::{Deserialize, Serialize};

use crate::purpose_rule::PurposeRule;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct SchemaVersion {
    version: NonZeroU32,
}

impl SchemaVersion {
    const FIRST_KEEPING_HOLDERS: Self = Self { version: NonZeroU32::MIN.saturating_add(6) };
    const PROJECTS: Self = Self::FIRST_KEEPING_HOLDERS.next();
    const PURPOSES: Self = Self::PROJECTS.next();

    const fn next(self) -> Self {
        Self { version: self.version.saturating_add(NonZeroU32::MIN.get()) }
    }

    #[must_use]
    pub fn keeps_holders_on_reopen(self) -> bool {
        self >= Self::FIRST_KEEPING_HOLDERS
    }

    #[must_use]
    pub fn purpose_rule(self) -> PurposeRule {
        if self >= Self::PURPOSES { PurposeRule::Required } else { PurposeRule::Optional }
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

#[cfg(test)]
mod tests {
    use super::SchemaVersion;
    use crate::purpose_rule::PurposeRule;

    #[test]
    fn the_current_version_does_not_yet_require_a_purpose_and_the_next_one_does() {
        let rules = [SchemaVersion::PROJECTS, SchemaVersion::PURPOSES].map(SchemaVersion::purpose_rule);
        assert_eq!(rules, [PurposeRule::Optional, PurposeRule::Required]);
        assert_eq!(SchemaVersion::current().purpose_rule(), PurposeRule::Optional);
        assert!(SchemaVersion::current() < SchemaVersion::PURPOSES);
    }
}
