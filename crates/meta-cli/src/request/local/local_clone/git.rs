use std::ffi::OsStr;
use std::fmt;
use std::path::Path;

use meta_run::Run;

use crate::request::cli_failure::{CliFailure, Setup};
use crate::request::project::CiSettings;

#[derive(Debug)]
pub struct Git<'dir> {
    dir: &'dir Path,
    settings: &'dir CiSettings,
}

impl<'dir> Git<'dir> {
    pub fn new(dir: &'dir Path, settings: &'dir CiSettings) -> Self {
        Self { dir, settings }
    }

    pub fn output(&self, words: &[&OsStr]) -> Result<String, CliFailure> {
        let ran = Run::git(self.dir).args(words).terminal().ran(self.settings.budget(CiSettings::CLAIM).deadline());
        let ran = ran.map_err(CliFailure::raised::<Setup>)?;
        ran.success().then(|| ran.stdout().text().trim().to_owned()).ok_or_else(|| {
            let line: Vec<_> = words.iter().map(|word| word.to_string_lossy()).collect();
            CliFailure::raised::<Setup>(format!("{self} {}: {}", line.join(" "), ran.stderr()))
        })
    }

    pub fn holds(&self, words: &[&OsStr]) -> Option<()> {
        self.output(words).ok().map(drop)
    }
}

impl fmt::Display for Git<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "git -C {}", self.dir.display())
    }
}
