mod renaming;
mod staged_name;

use std::fmt;
use std::io::{ErrorKind, Write as _};
use std::path::{Path, PathBuf};

use meta_door::kinds::Handle;

use meta_door::{Change, Door, Entry, Opening, Pair};

use crate::request::cli_failure::{CliFailure, Setup};
use renaming::Renaming;
use staged_name::StagedName;

#[derive(Debug)]
pub struct LocalFile(Entry);

impl LocalFile {
    const NEWLINE: char = '\n';

    pub fn new(path: impl Into<Entry>) -> Self {
        Self(path.into())
    }

    pub fn text(&self) -> Result<String, CliFailure> {
        Door::read(self.0.clone())
            .or_else(|failure| (failure.kind() == ErrorKind::NotFound).then(String::new).ok_or(failure))
            .map_err(|failure| self.failure(failure))
    }

    pub fn appended(&self, line: impl fmt::Display) -> Result<(), CliFailure> {
        let torn = self.text()?.chars().last().filter(|last| *last != Self::NEWLINE).map_or("", |_| "\n");
        self.opened(Opening::Append)
            .and_then(|mut file| writeln!(file, "{torn}{line}").map_err(|failure| self.failure(failure)))
    }

    pub fn opened(&self, opening: Opening) -> Result<Handle, CliFailure> {
        self.parent_made()?;
        Door::opened(self.0.clone(), opening).map_err(|failure| self.failure(failure))
    }

    pub fn replaced(&self, text: &str) -> Result<(), CliFailure> {
        let staged = Self(self.staging());
        self.parent_made()?;
        Door::write(staged.0.clone(), text).map_err(|failure| staged.failure(failure))?;
        Door::paired(staged.0.clone(), self.0.clone(), Pair::Rename).map_err(|failure| {
            Door::changed(staged.0.clone(), Change::RemoveFile).ok();
            staged.failure(Renaming::failed_onto(self.0.path(), failure))
        })
    }

    pub fn staging(&self) -> Entry {
        self.0.map(|path| path.with_file_name(StagedName::for_file(path.file_name().unwrap_or_default()).to_string()))
    }

    pub fn failure(&self, failure: impl fmt::Display) -> CliFailure {
        CliFailure::raised::<Setup>(format!("{self}: {failure}"))
    }

    fn parent_made(&self) -> Result<(), CliFailure> {
        let parent = self.0.map(|path| path.parent().map_or_else(PathBuf::new, Path::to_path_buf));
        let made = if parent.path().as_os_str().is_empty() { Ok(()) } else { Door::changed(parent, Change::DirAll) };
        made.map_err(|failure| self.failure(failure))
    }
}

impl fmt::Display for LocalFile {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.0)
    }
}
