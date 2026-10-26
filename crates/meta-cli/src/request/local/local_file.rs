use std::fmt;
use std::io::{ErrorKind, Write as _};
use std::path::PathBuf;

use crate::request::cli_failure::{CliFailure, Setup};

#[derive(Debug)]
pub struct LocalFile(PathBuf);

impl LocalFile {
    pub fn new(path: PathBuf) -> Self {
        Self(path)
    }

    pub fn text(&self) -> Result<String, CliFailure> {
        std::fs::read_to_string(&self.0)
            .or_else(|failure| (failure.kind() == ErrorKind::NotFound).then(String::new).ok_or(failure))
            .map_err(|failure| self.failure(failure))
    }

    pub fn appended(&self, line: impl fmt::Display) -> Result<(), CliFailure> {
        self.0.parent().map_or(Ok(()), std::fs::create_dir_all).map_err(|failure| self.failure(failure))?;
        std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.0)
            .and_then(|mut file| writeln!(file, "{line}"))
            .map_err(|failure| self.failure(failure))
    }

    pub fn failure(&self, failure: impl fmt::Display) -> CliFailure {
        CliFailure::raised::<Setup>(format!("{self}: {failure}"))
    }
}

impl fmt::Display for LocalFile {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.0.display())
    }
}
