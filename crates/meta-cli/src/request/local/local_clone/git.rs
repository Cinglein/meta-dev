use std::ffi::OsStr;
use std::fmt;
use std::path::Path;
use std::time::Instant;

use meta_run::{KillScope, Run};

use crate::request::cli_failure::{CliFailure, Setup};
use crate::request::project::CiSettings;

#[derive(Clone, Copy, Debug)]
pub struct Git<'dir> {
    dir: &'dir Path,
    settings: &'dir CiSettings,
    scope: KillScope,
}

impl<'dir> Git<'dir> {
    pub fn new(dir: &'dir Path, settings: &'dir CiSettings) -> Self {
        Self { dir, settings, scope: KillScope::Process }
    }

    pub fn reaching(self) -> Self {
        Self { scope: KillScope::Group, ..self }
    }

    pub fn output(&self, words: &[&OsStr]) -> Result<String, CliFailure> {
        let run = Run::git(self.dir).args(words).scoped(self.scope);
        let ran = run.ran(self.settings.git_deadline(Instant::now()));
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
