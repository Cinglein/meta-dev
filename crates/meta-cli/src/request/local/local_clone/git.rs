use std::ffi::OsStr;
use std::fmt;
use std::path::Path;
use std::process::Command;

use meta_trace::SpawnSpan;

use crate::request::cli_failure::{CliFailure, Setup};

#[derive(Debug)]
pub struct Git<'dir>(&'dir Path);

impl<'dir> Git<'dir> {
    pub fn new(dir: &'dir Path) -> Self {
        Self(dir)
    }

    pub fn output(&self, words: &[&OsStr]) -> Result<String, CliFailure> {
        let output = SpawnSpan::output_of(Command::new("git").arg("-C").arg(self.0).args(words))
            .map_err(CliFailure::raised::<Setup>)?;
        let shown = |stream: &[u8]| String::from_utf8_lossy(stream).trim().to_owned();
        output.status.success().then(|| shown(&output.stdout)).ok_or_else(|| {
            let line: Vec<_> = words.iter().map(|word| word.to_string_lossy()).collect();
            CliFailure::raised::<Setup>(format!("{self} {}: {}", line.join(" "), shown(&output.stderr)))
        })
    }

    pub fn holds(&self, words: &[&OsStr]) -> Option<()> {
        self.output(words).ok().map(drop)
    }
}

impl fmt::Display for Git<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "git -C {}", self.0.display())
    }
}
