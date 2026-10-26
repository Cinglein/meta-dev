use std::fmt;

use serde::{Deserialize, Serialize};

use crate::division::Division;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Placement(Option<Division>);

impl Placement {
    #[must_use]
    pub fn new(division: Division) -> Self {
        Self(Some(division))
    }

    #[must_use]
    pub fn division(&self) -> Option<&Division> {
        self.0.as_ref()
    }

    #[must_use]
    pub fn is_root(&self) -> bool {
        self.0.is_none()
    }
}

impl fmt::Display for Placement {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.as_ref().map_or(Ok(()), |division| write!(formatter, " @{division}"))
    }
}
