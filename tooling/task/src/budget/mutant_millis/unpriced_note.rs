use std::fmt;

use crate::budget::Millis;
use crate::crate_name::CrateName;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UnpricedNote {
    name: CrateName,
    planned: Millis,
}

impl UnpricedNote {
    #[must_use]
    pub fn new(name: CrateName, planned: Millis) -> UnpricedNote {
        UnpricedNote { name, planned }
    }
}

impl fmt::Display for UnpricedNote {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "mutants: {} is unpriced in [mutant_millis]; planned at the table median, {} ms per mutant; `cargo xtask mutant-costs` will measure it",
            self.name, self.planned
        )
    }
}
