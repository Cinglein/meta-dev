use std::fmt;
use std::path::PathBuf;

use crate::line_number::LineNumber;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Site {
    Workspace,
    File(PathBuf),
    Line(PathBuf, LineNumber),
}

impl fmt::Display for Site {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Site::Workspace => formatter.write_str("workspace"),
            Site::File(path) => write!(formatter, "{}", path.display()),
            Site::Line(path, line) => write!(formatter, "{}:{line}", path.display()),
        }
    }
}
