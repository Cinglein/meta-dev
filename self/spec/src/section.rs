use std::fmt;

use crate::rule::Rule;
use crate::wording::Wording;

#[derive(Clone, Copy, Debug)]
pub struct Section {
    heading: Wording,
    rules: &'static [Rule],
}

impl Section {
    #[must_use]
    pub const fn new(heading: &'static str, rules: &'static [Rule]) -> Section {
        Section { heading: Wording::new(heading), rules }
    }

    #[must_use]
    pub fn rules(&self) -> &'static [Rule] {
        self.rules
    }
}

impl fmt::Display for Section {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(formatter, "## {}", self.heading)?;
        self.rules.iter().try_for_each(|rule| writeln!(formatter, "{rule}"))?;
        writeln!(formatter)
    }
}
