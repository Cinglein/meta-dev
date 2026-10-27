use derive_more::{Display, From, FromStr};
use serde::{Deserialize, Serialize};

#[derive(
    Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Display, From, FromStr, Serialize, Deserialize,
)]
#[serde(transparent)]
pub struct BudgetId(String);
