use std::fmt;

use super::expectation::Expectation;

pub struct Mismatch {
    missing: Vec<Expectation>,
    unexpected: Vec<Expectation>,
}

impl Mismatch {
    pub fn between(expected: Vec<Expectation>, found: Vec<Expectation>) -> Mismatch {
        found.into_iter().fold(Mismatch { missing: expected, unexpected: Vec::new() }, Mismatch::absorbing)
    }

    pub fn is_empty(&self) -> bool {
        self.missing.is_empty() && self.unexpected.is_empty()
    }

    fn absorbing(mut self, found: Expectation) -> Mismatch {
        let matched =
            self.missing.iter().position(|expected| expected.accepts(&found)).map(|index| self.missing.remove(index));
        self.unexpected.extend(matched.is_none().then_some(found));
        self
    }

    fn listed(formatter: &mut fmt::Formatter<'_>, headline: &str, entries: &[Expectation]) -> fmt::Result {
        let mut sorted: Vec<&Expectation> = entries.iter().collect();
        sorted.sort();
        writeln!(formatter, "{headline} ({}):", entries.len())?;
        sorted.iter().try_for_each(|entry| writeln!(formatter, "  {entry}"))
    }
}

impl fmt::Display for Mismatch {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        Self::listed(formatter, "marked but not reported", &self.missing)?;
        Self::listed(formatter, "reported but not marked", &self.unexpected)
    }
}
