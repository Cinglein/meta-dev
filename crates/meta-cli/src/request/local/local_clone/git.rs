use std::ffi::OsStr;
use std::fmt;
use std::path::Path;

use meta_run::{GitArgv, Port, Tool as _};

use super::LocalClone;
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

    pub fn output(&self, call: GitArgv, operands: &[&OsStr]) -> Result<String, CliFailure> {
        let deadline = self.settings.budget(CiSettings::CLAIM).deadline();
        let reported = Port::<GitArgv>::new(self.dir).run(call, operands).terminal().reported(deadline);
        reported.map(|shown| shown.text().trim().to_owned()).map_err(|unread| {
            let words = call.words().iter().map(OsStr::new).chain(operands.iter().copied());
            let line: Vec<_> = words.map(OsStr::to_string_lossy).collect();
            CliFailure::raised::<Setup>(format!("{self} {}: {unread}", line.join(" ")))
        })
    }
}

impl<'dir> From<&'dir LocalClone> for Git<'dir> {
    fn from(clone: &'dir LocalClone) -> Self {
        Self::new(&clone.checkout, &clone.settings)
    }
}

impl fmt::Display for Git<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "git -C {}", self.dir.display())
    }
}
