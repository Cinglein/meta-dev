use std::fmt;

use crate::git_log::remote::Remote;
use crate::git_log::sha::Sha;

#[derive(Debug)]
pub enum BatchCommand<'tip> {
    TreeInfo(&'tip Sha),
    FileContents(&'tip Sha),
}

impl fmt::Display for BatchCommand<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TreeInfo(tip) => writeln!(formatter, "info {tip}^{{tree}}"),
            Self::FileContents(tip) => writeln!(formatter, "contents {tip}:{}", Remote::FILE),
        }
    }
}
