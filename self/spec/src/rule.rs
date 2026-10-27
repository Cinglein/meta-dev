mod rule_id;

use std::fmt;

use crate::enforcement::Enforcement;
use crate::wording::Wording;
pub use rule_id::RuleId;
use task::{SpecSettings, TopDivision};

#[derive(Clone, Copy, Debug)]
pub struct Rule {
    id: RuleId,
    division: TopDivision,
    wording: Wording,
    enforcement: Enforcement,
}

impl Rule {
    #[must_use]
    pub const fn new(id: RuleId, division: TopDivision, wording: Wording, enforcement: Enforcement) -> Rule {
        Rule { id, division, wording, enforcement }
    }

    #[must_use]
    pub fn rule_id(&self) -> RuleId {
        self.id
    }

    #[must_use]
    pub fn enforcement(&self) -> Enforcement {
        self.enforcement
    }
}

impl fmt::Display for Rule {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{}{}` ({}, {}): {}",
            SpecSettings::RULE,
            self.id,
            self.division,
            self.enforcement,
            self.wording
        )
    }
}
