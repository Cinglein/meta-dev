mod amount;
mod part;

use serde::{Deserialize, Serialize};

use amount::Amount;
use part::Part;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Allowance {
    Whole(Amount),
    PartOf(Part),
}

impl Allowance {
    #[must_use]
    pub fn part(&self) -> Option<&Part> {
        match self {
            Allowance::PartOf(part) => Some(part),
            Allowance::Whole(_) => None,
        }
    }
}
