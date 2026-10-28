use std::fmt;

use super::line::Line;

#[derive(Debug)]
pub struct Leak {
    commit: String,
    file: String,
    line: Line,
}

impl Leak {
    pub fn new(commit: &str, file: &str, line: Line) -> Self {
        Self { commit: commit.to_owned(), file: file.to_owned(), line }
    }
}

impl fmt::Display for Leak {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{}:{}: agent scratch path in commit {}; remove it from that commit (amend or recommit; a later fix commit does not clear it); not pushed",
            self.file, self.line, self.commit
        )
    }
}
