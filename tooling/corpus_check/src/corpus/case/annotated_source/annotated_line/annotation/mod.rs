mod scope;

use std::fmt;
use std::path::Path;
use std::str::FromStr;

use scope::Scope;

use super::super::super::super::corpus_error::CorpusError;
use super::super::super::super::expectation::Expectation;
use crate::lint_name::LintName;
use task::LineNumber;
use task::Name;

#[derive(Debug, PartialEq, Eq)]
pub struct Annotation {
    scope: Scope,
    lint: LintName,
    message: String,
}

impl Annotation {
    pub fn placed(&self, case: &Name, path: &Path, line: LineNumber) -> Expectation {
        Expectation::marked(case, self.lint, self.scope.site(path, line), &self.message)
    }
}

impl FromStr for Annotation {
    type Err = CorpusError;

    fn from_str(text: &str) -> Result<Annotation, CorpusError> {
        let malformed = || CorpusError::Marker { text: text.to_owned() };
        let (head, message) = text.split_once(':').ok_or_else(malformed)?;
        let words: Vec<&str> = head.split_whitespace().collect();
        let scope = match words.as_slice() {
            [_] => Ok(Scope::Line),
            [scope, _] => scope.parse().map_err(|_| malformed()),
            _ => Err(malformed()),
        }?;
        Ok(Annotation {
            scope,
            lint: words.last().ok_or_else(malformed)?.parse().map_err(|_| malformed())?,
            message: message.trim().to_owned(),
        })
    }
}

impl fmt::Display for Annotation {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.scope {
            Scope::Line => write!(formatter, "//~ {}: {}", self.lint, self.message),
            scoped => write!(formatter, "//~ {scoped} {}: {}", self.lint, self.message),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Annotation;
    const MARKERS: [&str; 3] = [
        "//~ no-free-fns: fn helper",
        "//~ file module-nesting: used only from crate::hub",
        "//~ workspace test-budget: has 7 tests",
    ];

    #[test]
    fn a_marker_names_a_lint_and_a_message_with_an_optional_file_or_workspace_scope() {
        let round_tripped =
            MARKERS.map(|marker| marker.trim_start_matches("//~").parse::<Annotation>().unwrap().to_string());
        assert_eq!(round_tripped, MARKERS);
        assert!(" no colon".parse::<Annotation>().is_err());
    }
}
