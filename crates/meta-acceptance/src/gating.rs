use serde::de::Error as _;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::acceptance_failure::AcceptanceFailure;
use crate::enforcement::Enforcement;
use crate::vec1::Vec1;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Gating(Vec1<Enforcement>);

impl Gating {
    #[must_use]
    pub fn points(&self) -> &Vec1<Enforcement> {
        &self.0
    }

    fn checked(points: Vec1<Enforcement>) -> Result<Self, AcceptanceFailure> {
        points
            .as_slice()
            .iter()
            .any(|point| !matches!(point, Enforcement::Hook(_)))
            .then_some(Self(points))
            .ok_or(AcceptanceFailure::Ungated)
    }
}

impl TryFrom<Vec<Enforcement>> for Gating {
    type Error = AcceptanceFailure;

    fn try_from(points: Vec<Enforcement>) -> Result<Self, Self::Error> {
        Vec1::try_from(points).and_then(Self::checked)
    }
}

impl Serialize for Gating {
    fn serialize<Out: Serializer>(&self, serializer: Out) -> Result<Out::Ok, Out::Error> {
        self.0.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Gating {
    fn deserialize<In: Deserializer<'de>>(deserializer: In) -> Result<Self, In::Error> {
        Self::checked(Vec1::<Enforcement>::deserialize(deserializer)?).map_err(In::Error::custom)
    }
}
