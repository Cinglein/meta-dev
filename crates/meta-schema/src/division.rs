mod division_failure;
mod division_name;

use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::top_division::TopDivision;
use crate::unparsed::Unparsed;
use division_failure::DivisionFailure;
use division_name::DivisionName;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct Division {
    top: TopDivision,
    below: Vec<DivisionName>,
}

impl Division {
    #[must_use]
    pub fn strictly_contains(&self, inner: &Division) -> bool {
        self.top == inner.top && inner.below.len() > self.below.len() && inner.below.starts_with(&self.below)
    }

    #[must_use]
    pub fn is_nested(&self) -> bool {
        !self.below.is_empty()
    }
}

impl FromStr for Division {
    type Err = DivisionFailure;

    fn from_str(id: &str) -> Result<Division, DivisionFailure> {
        let refused = || DivisionFailure::Unplaced(Unparsed::from(id));
        let (top, below) = id.split_once('/').map_or((id, None), |(top, below)| (top, Some(below)));
        let below = below.map_or(Some(Vec::new()), |below| below.split('/').map(DivisionName::named).collect());
        match (top.parse(), below) {
            (Ok(top), Some(below)) => Ok(Division { top, below }),
            _ => Err(refused()),
        }
    }
}

impl TryFrom<String> for Division {
    type Error = DivisionFailure;

    fn try_from(id: String) -> Result<Division, DivisionFailure> {
        id.parse()
    }
}

impl From<Division> for String {
    fn from(division: Division) -> String {
        division.to_string()
    }
}

impl fmt::Display for Division {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.top)?;
        self.below.iter().try_for_each(|name| write!(formatter, "/{name}"))
    }
}

#[cfg(test)]
mod tests {
    use super::Division;

    const TOP: &str = "performance-and-cost";
    const NESTED: &str = "performance-and-cost/ci/tests";
    const REFUSED: [&str; 3] = ["performance", "performance-and-cost//ci", "features-and-maintenance/"];

    #[test]
    fn a_division_id_starts_at_a_top_division_and_nests_by_slashes() {
        let [top, nested] = [TOP, NESTED].map(|id| id.parse::<Division>().unwrap());
        assert_eq!(nested.to_string(), NESTED);
        assert!(top.strictly_contains(&nested) && !nested.strictly_contains(&top) && !top.strictly_contains(&top));
        assert!(REFUSED.iter().all(|id| id.parse::<Division>().is_err()));
    }
}
