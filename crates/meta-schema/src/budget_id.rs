use std::convert::Infallible;
use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct BudgetId(String);

impl From<String> for BudgetId {
    fn from(id: String) -> BudgetId {
        BudgetId(id)
    }
}

impl FromStr for BudgetId {
    type Err = Infallible;

    fn from_str(id: &str) -> Result<BudgetId, Infallible> {
        Ok(BudgetId(id.to_owned()))
    }
}

impl fmt::Display for BudgetId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}
