use serde::{Deserialize, Serialize};

use crate::budget::Millis;
use crate::policy::most::Most;
use crate::policy::percent::Percent;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub enum Amount {
    Count(Most),
    Duration(Millis),
    Share(Percent),
}
