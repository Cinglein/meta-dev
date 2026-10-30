use std::fmt;
use std::path::{Path, PathBuf};

use crate::line_number::LineNumber;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Site {
    Workspace,
    File(PathBuf),
    Line(PathBuf, LineNumber),
}

impl Site {
    #[must_use]
    pub fn path(&self) -> Option<&Path> {
        match self {
            Site::Workspace => None,
            Site::File(path) | Site::Line(path, _) => Some(path),
        }
    }
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

#[cfg(test)]
mod tests {
    use super::{LineNumber, PathBuf, Site};
    #[test]
    fn a_line_site_prints_as_its_path_and_line_number() {
        let line = Site::Line(PathBuf::from("a.rs"), LineNumber::new(3));
        assert_eq!(line.to_string(), "a.rs:3");
    }
}
