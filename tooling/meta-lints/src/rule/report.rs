use std::fmt;

use crate::violation::Violation;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Report {
    headline: &'static str,
    violations: Vec<Violation>,
}

impl Report {
    pub fn new(headline: &'static str, violations: Vec<Violation>) -> Report {
        Report { headline, violations }
    }

    pub fn violations(&self) -> &[Violation] {
        &self.violations
    }

    pub fn headline(&self) -> &'static str {
        self.headline
    }
}

impl fmt::Display for Report {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(formatter, "{}:", self.headline)?;
        self.violations.iter().try_for_each(|violation| writeln!(formatter, "{violation}"))
    }
}
