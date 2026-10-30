use std::fmt;
use std::str::FromStr;

use serde::de::Error as _;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::vocabulary_failure::VocabularyFailure;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct BudgetId(String);

impl From<&'static str> for BudgetId {
    fn from(id: &'static str) -> BudgetId {
        BudgetId(id.to_owned())
    }
}

impl FromStr for BudgetId {
    type Err = VocabularyFailure;

    fn from_str(id: &str) -> Result<BudgetId, VocabularyFailure> {
        (!id.is_empty()).then(|| BudgetId(id.to_owned())).ok_or(VocabularyFailure::EmptyBudget)
    }
}

impl fmt::Display for BudgetId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl Serialize for BudgetId {
    fn serialize<Out: Serializer>(&self, serializer: Out) -> Result<Out::Ok, Out::Error> {
        serializer.serialize_str(&self.0)
    }
}

impl<'de> Deserialize<'de> for BudgetId {
    fn deserialize<In: Deserializer<'de>>(deserializer: In) -> Result<Self, In::Error> {
        String::deserialize(deserializer)?.parse().map_err(In::Error::custom)
    }
}
