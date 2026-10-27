use std::fmt;

use serde::{Deserialize, Serialize};

use crate::change::Change;
use crate::division::Division;
use crate::placement::Placement;
use crate::target::Target;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Relocation {
    item: Target,
    #[serde(default, skip_serializing_if = "Placement::is_root")]
    from: Placement,
    division: Division,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    change: Option<Change>,
}

impl Relocation {
    #[must_use]
    pub fn new(item: Target, from: Placement, division: Division, change: Option<Change>) -> Relocation {
        Relocation { item, from, division, change }
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
    pub fn change(&self) -> Option<&Change> {
        self.change.as_ref()
    }
}

impl fmt::Display for Relocation {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}{} to @{}", self.item, self.from, self.division)?;
        self.change.as_ref().map_or(Ok(()), |change| write!(formatter, " !{change}"))
    }
}
