use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use super::kept::Kept;
use crate::request::cli_failure::Message;

#[derive(Debug)]
pub enum Occupancy {
    Listed(Vec<PathBuf>),
    Unlisted(Message),
}

impl Occupancy {
    const NAMED: char = 'n';
    const RECURSIVE: &'static str = "+D";

    pub fn scanned(paths: &[PathBuf]) -> Self {
        Command::new("lsof")
            .args(["-w", "-F", "n"])
            .args(paths.iter().flat_map(|path| [OsStr::new(Self::RECURSIVE), path.as_os_str()]))
            .output()
            .map_or_else(|failure| Self::Unlisted(Message::new(failure)), |listed| Self::read(&listed))
    }

    pub fn vacant(&self, path: &Path) -> Result<(), Kept> {
        match self {
            Self::Listed(open) if open.iter().any(|name| name.starts_with(path)) => Err(Kept::InUse),
            Self::Listed(_) => Ok(()),
            Self::Unlisted(cause) => Err(Kept::Uninspected(Message::new(cause))),
        }
    }

    fn read(listed: &Output) -> Self {
        if listed.stderr.is_empty() {
            Self::Listed(
                String::from_utf8_lossy(&listed.stdout)
                    .lines()
                    .filter_map(|line| line.strip_prefix(Self::NAMED))
                    .map(PathBuf::from)
                    .collect(),
            )
        } else {
            Self::Unlisted(Message::new(String::from_utf8_lossy(&listed.stderr)))
        }
    }
}
