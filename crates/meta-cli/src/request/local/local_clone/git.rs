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

    pub fn holds(&self, call: GitArgv, operands: &[&OsStr]) -> Option<()> {
        self.output(call, operands).ok().map(drop)
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

#[cfg(test)]
mod tests {
    use std::ffi::OsStr;
    use std::path::Path;

    use meta_run::GitArgv;

    use super::Git;
    use crate::request::project::{CiSettings, Config};

    const NOWHERE: &str = "/nonexistent/clone";
    const HEAD: &str = "HEAD";
    const SHOWN: &str = "git -C /nonexistent/clone rev-parse --verify --quiet HEAD: ";

    #[test]
    fn a_failed_call_names_its_directory_and_whole_argv_and_holds_nothing() {
        let settings = CiSettings::try_from(&Config::default()).unwrap();
        let git = Git::new(Path::new(NOWHERE), &settings);
        let failed = git.output(GitArgv::VerifyRef, &[OsStr::new(HEAD)]).map_err(|failure| failure.to_string());
        assert!(
            failed.is_err_and(|failure| failure.starts_with(SHOWN)) && git.holds(GitArgv::VerifyRef, &[]).is_none()
        );
    }
}
