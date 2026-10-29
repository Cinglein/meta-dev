use std::fmt;

use settings::EmbeddedDefaults;

use crate::budgets::LegacyKey;
use crate::policy::Policy;

#[derive(Clone, Copy, Debug)]
pub struct DefaultBudget(LegacyKey);

impl From<LegacyKey> for DefaultBudget {
    fn from(key: LegacyKey) -> DefaultBudget {
        DefaultBudget(key)
    }
}

impl DefaultBudget {
    const BLOCK_END: &str = "\n\n";
}

impl fmt::Display for DefaultBudget {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let header = format!("[budgets.{}]\n", self.0.budget_id());
        let rest = Policy::DEFAULTS
            .split_once(&header)
            .map_or("", |(_, rest)| rest);
        let block = rest
            .split_once(Self::BLOCK_END)
            .map_or(rest, |(block, _)| block);
        write!(formatter, "{header}{block}")
    }
}
