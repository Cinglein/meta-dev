mod rule_id;

use std::fmt;

use crate::division::Division;
use crate::enforcement::Enforcement;
use crate::wording::Wording;
use rule_id::RuleId;

#[derive(Clone, Copy, Debug)]
pub struct Rule {
    id: RuleId,
    division: Division,
    wording: Wording,
    enforcement: Enforcement,
}

impl Rule {
    #[must_use]
    pub const fn new(id: &'static str, division: Division, wording: &'static str, enforcement: Enforcement) -> Rule {
        Rule { id: RuleId::new(id), division, wording: Wording::new(wording), enforcement }
    }

    #[must_use]
    pub fn rule_id(&self) -> RuleId {
        self.id
    }

    #[must_use]
    pub fn enforcement(&self) -> &Enforcement {
        &self.enforcement
    }
}

impl fmt::Display for Rule {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "- `{}` ({}, {}): {}", self.id, self.division, self.enforcement, self.wording)
    }
}
