use std::fmt;

use serde::{Deserialize, Serialize};

use crate::carry::Carry;
use crate::division::Division;
use crate::placement::Placement;
use crate::target::Target;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Relocation {
    item: Target,
    #[serde(default, skip_serializing_if = "Placement::is_root")]
    from: Placement,
    division: Division,
    carry: Carry,
}

impl Relocation {
    #[must_use]
    pub fn new(item: Target, from: Placement, division: Division, carry: Carry) -> Relocation {
        Relocation { item, from, division, carry }
    }

    #[must_use]
    pub fn target(&self) -> Target {
        self.item
    }

    #[must_use]
    pub fn origin(&self) -> &Placement {
        &self.from
    }

    #[must_use]
    pub fn division(&self) -> &Division {
        &self.division
    }

    #[must_use]
    pub fn carry(&self) -> &Carry {
        &self.carry
    }
}

impl fmt::Display for Relocation {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}{} to @{}{}", self.item, self.from, self.division, self.carry)
    }
}
