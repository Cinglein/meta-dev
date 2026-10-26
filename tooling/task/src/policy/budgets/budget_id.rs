use derive_more::{Display, FromStr};
use serde::{Deserialize, Serialize};

#[derive(
    Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Display, FromStr, Serialize, Deserialize,
)]
#[serde(transparent)]
pub struct BudgetId(String);
