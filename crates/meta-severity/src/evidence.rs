use std::cmp::Ordering;

use meta_schema::Scenario;

use crate::rating_failure::RatingFailure;
use crate::reading::Reading;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Evidence {
    Absent,
    Measured,
    FailedRemedy,
    Scenario(Scenario),
}

impl Evidence {
    pub fn admits(&self, from: &Reading, rise: Ordering) -> Result<Ordering, RatingFailure> {
        match (rise, self) {
            (Ordering::Less, Self::Absent) => Err(RatingFailure::RaisedWithoutEvidence(from.clone())),
            _ => Ok(rise),
        }
    }
}

impl From<Option<Scenario>> for Evidence {
    fn from(scenario: Option<Scenario>) -> Evidence {
        scenario.map_or(Evidence::Absent, Evidence::Scenario)
    }
}
