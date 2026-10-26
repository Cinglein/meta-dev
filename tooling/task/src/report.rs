use std::fmt;

use crate::failure::Failure;
use crate::violation::Violation;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Report {
    headline: &'static str,
    violations: Vec<Violation>,
}

impl Report {
    #[must_use]
    pub fn new(headline: &'static str, violations: Vec<Violation>) -> Report {
        Report {
            headline,
            violations,
        }
    }

    #[must_use]
    pub fn is_clean(&self) -> bool {
        self.violations.is_empty()
    }

    #[must_use]
    pub fn lists_exactly(&self, expected: &[&str]) -> bool {
        self.violations.len() == expected.len()
            && self
                .violations
                .iter()
                .zip(expected)
                .all(|(violation, text)| violation.to_string().contains(text))
    }

    pub fn verdict(self) -> Result<(), Failure> {
        if self.is_clean() {
            Ok(())
        } else {
            Err(Failure::Lint(vec![self]))
        }
    }
}

impl fmt::Display for Report {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(formatter, "{}:", self.headline)?;
        self.violations
            .iter()
            .try_for_each(|violation| writeln!(formatter, "{violation}"))
    }
}

#[cfg(test)]
mod tests {
    use super::Report;
    use crate::site::Site;
    use crate::violation::Violation;
    const HEADLINE: &str = "something slipped";
    const MESSAGE: &str = "a slip";

    #[test]
    fn a_report_is_clean_only_without_violations_and_lists_exactly_what_it_holds() {
        let clean = Report::new(HEADLINE, Vec::new());
        let dirty = Report::new(HEADLINE, vec![Violation::new(Site::Workspace, MESSAGE)]);
        assert!(
            clean.is_clean()
                && !dirty.is_clean()
                && clean.clone().verdict().is_ok()
                && dirty.clone().verdict().is_err()
        );
        assert!(
            dirty.lists_exactly(&[MESSAGE])
                && !dirty.lists_exactly(&[])
                && !dirty.lists_exactly(&[MESSAGE, MESSAGE])
        );
        assert!(dirty.to_string().contains(HEADLINE) && dirty.to_string().contains(MESSAGE));
    }
}
