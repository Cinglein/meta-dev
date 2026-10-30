use std::collections::BTreeSet;

use serde::de::Error as _;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::acceptance_failure::AcceptanceFailure;
use crate::acceptance_row::AcceptanceRow;
use crate::vec1::Vec1;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AcceptanceRows(Vec1<AcceptanceRow>);

impl AcceptanceRows {
    #[must_use]
    pub fn rows(&self) -> &Vec1<AcceptanceRow> {
        &self.0
    }

    fn distinct(rows: Vec1<AcceptanceRow>) -> Result<Self, AcceptanceFailure> {
        let mut seen = BTreeSet::new();
        match rows.as_slice().iter().find(|row| !seen.insert(row.budget())) {
            Some(row) => Err(AcceptanceFailure::DuplicateBudget(row.budget().clone())),
            None => Ok(Self(rows)),
        }
    }
}

impl TryFrom<Vec<AcceptanceRow>> for AcceptanceRows {
    type Error = AcceptanceFailure;

    fn try_from(rows: Vec<AcceptanceRow>) -> Result<Self, Self::Error> {
        Vec1::try_from(rows).and_then(Self::distinct)
    }
}

impl Serialize for AcceptanceRows {
    fn serialize<Out: Serializer>(&self, serializer: Out) -> Result<Out::Ok, Out::Error> {
        self.0.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for AcceptanceRows {
    fn deserialize<In: Deserializer<'de>>(deserializer: In) -> Result<Self, In::Error> {
        Self::distinct(Vec1::<AcceptanceRow>::deserialize(deserializer)?).map_err(In::Error::custom)
    }
}
