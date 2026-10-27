use serde::{Deserialize, Serialize};

use crate::budget::Millis;
use crate::policy::budgets::budget::measure::Unit;
use crate::policy::most::Most;
use crate::policy::percent::Percent;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub enum Amount {
    Count(Most),
    Duration(Millis),
    Share(Percent),
}

impl Amount {
    #[must_use]
    pub const fn unit(self) -> Unit {
        match self {
            Amount::Count(_) => Unit::Count,
            Amount::Duration(_) => Unit::Duration,
            Amount::Share(_) => Unit::Share,
        }
    }
}
