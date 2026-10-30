use std::fmt;

use crate::file_line::FileLine;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Addition {
    file: String,
    line: FileLine,
    commit: String,
}

impl Addition {
    #[must_use]
    pub fn new(file: String, line: FileLine, commit: String) -> Addition {
        Addition { file, line, commit }
    }

    #[must_use]
    pub fn commit(&self) -> &str {
        &self.commit
    }
}

impl fmt::Display for Addition {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}:{}", self.file, self.line)
    }
}
