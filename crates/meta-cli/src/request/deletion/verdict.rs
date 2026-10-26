use std::fmt;
use std::path::PathBuf;

use super::kept::Kept;
use crate::request::output::Output;

#[derive(Debug)]
pub enum Verdict {
    Removed(Output),
    Refused(PathBuf, Kept),
}

impl fmt::Display for Verdict {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Removed(logged) => write!(formatter, "{logged}"),
            Self::Refused(path, kept) => writeln!(formatter, "kept {}: {kept}", path.display()),
        }
    }
}
