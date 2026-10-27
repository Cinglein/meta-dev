use std::fmt;

use serde::{Deserialize, Serialize};

use crate::change::Change;
use crate::division::Division;
use crate::target::Target;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Rerating {
    item: Target,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    division: Option<Division>,
    change: Change,
}

impl Rerating {
    #[must_use]
    pub fn new(item: Target, division: Division, change: Change) -> Rerating {
        Rerating { item, division: Some(division), change }
    }

    #[must_use]
    pub fn target(&self) -> Target {
        self.item
    }

    #[must_use]
    pub fn seen_in(&self) -> Option<&Division> {
        self.division.as_ref()
    }

    #[must_use]
    pub fn change(&self) -> &Change {
        &self.change
    }
}

impl fmt::Display for Rerating {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{} {}", self.item, self.change)
    }
}
