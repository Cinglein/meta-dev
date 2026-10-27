use std::fmt;

use serde::{Deserialize, Serialize};

use crate::change::Change;
use crate::target::Target;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Rerating {
    item: Target,
    change: Change,
}

impl Rerating {
    #[must_use]
    pub fn new(item: Target, change: Change) -> Rerating {
        Rerating { item, change }
    }

    #[must_use]
    pub fn target(&self) -> Target {
        self.item
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
