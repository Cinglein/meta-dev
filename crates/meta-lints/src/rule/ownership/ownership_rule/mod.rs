mod stray;

use crate::path_prefix::PathPrefix;
use serde::Deserialize;
use stray::Stray;

use crate::line_number::LineNumber;
use crate::site::Site;
use crate::source_file::SourceFile;
use crate::violation::Violation;

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OwnershipRule {
    owner: PathPrefix,
    #[serde(default)]
    within: PathPrefix,
    tokens: Vec<String>,
}

impl OwnershipRule {
    pub fn violations(&self, file: &SourceFile) -> Vec<Violation> {
        if self.within.covers(file.path()) && !self.owner.covers(file.path()) {
            file.text()
                .lines()
                .enumerate()
                .filter_map(|(index, line)| self.stray(file.path(), LineNumber::after_index(index), line))
                .collect()
        } else {
            Vec::new()
        }
    }

    fn stray(&self, path: &str, line_number: LineNumber, line: &str) -> Option<Violation> {
        self.tokens
            .iter()
            .find(|token| line.contains(token.as_str()))
            .map(|token| Violation::new(Site::Line(path.to_owned(), line_number), Stray::new(token, &self.owner)))
    }
}
