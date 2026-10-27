mod renaming;
mod staged_name;

use std::fmt;
use std::io::{ErrorKind, Write as _};
use std::path::PathBuf;

use crate::request::cli_failure::{CliFailure, Setup};
use renaming::Renaming;
use staged_name::StagedName;

#[derive(Debug)]
pub struct LocalFile(PathBuf);

impl LocalFile {
    const NEWLINE: char = '\n';

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
        let torn = self.text()?.chars().last().filter(|last| *last != Self::NEWLINE).map_or("", |_| "\n");
        std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.0)
            .and_then(|mut file| writeln!(file, "{torn}{line}"))
            .map_err(|failure| self.failure(failure))
    }

    pub fn replaced(&self, text: &str) -> Result<(), CliFailure> {
        let staged = Self::new(self.staging());
        self.0.parent().map_or(Ok(()), std::fs::create_dir_all).map_err(|failure| self.failure(failure))?;
        std::fs::write(&staged.0, text).map_err(|failure| staged.failure(failure))?;
        std::fs::rename(&staged.0, &self.0).map_err(|failure| {
            std::fs::remove_file(&staged.0).ok();
            staged.failure(Renaming::failed_onto(&self.0, failure))
        })
    }

    pub fn staging(&self) -> PathBuf {
        self.0.with_file_name(StagedName::for_file(self.0.file_name().unwrap_or_default()).to_string())
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
