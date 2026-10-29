use std::fmt;

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct IntentCap(usize);

impl IntentCap {
    #[must_use]
    pub fn allows(self, found: usize) -> bool {
        found <= self.0
    }
}

impl fmt::Display for IntentCap {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.0)
    }
}
