use std::fmt;

use crate::lint_name::LintName;
use task::Name;
use task::Site;
use task::Violation;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Expectation {
    case: Name,
    lint: LintName,
    site: Site,
    message: String,
}

impl Expectation {
    pub fn marked(case: &Name, lint: LintName, site: Site, message: &str) -> Expectation {
        Expectation { case: case.clone(), lint, site, message: message.to_owned() }
    }

    pub fn found(case: &Name, lint: LintName, violation: &Violation) -> Expectation {
        Expectation {
            case: case.clone(),
            lint,
            site: violation.site().clone(),
            message: violation.message().to_owned(),
        }
    }

    pub fn accepts(&self, found: &Expectation) -> bool {
        self.case == found.case
            && self.lint == found.lint
            && self.site == found.site
            && found.message.contains(&self.message)
    }

    pub fn is_for(&self, lint: LintName) -> bool {
        self.lint == lint
    }
}

impl fmt::Display for Expectation {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}/{}: {}: {}", self.case, self.site, self.lint, self.message)
    }
}
