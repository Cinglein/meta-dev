use std::ffi::OsStr;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use meta_run::{Ran, Run};

use crate::log_failure::LogFailure;

#[derive(Clone, Debug)]
pub struct Git {
    repo: PathBuf,
}

impl Git {
    const LIMIT: Duration = Duration::from_secs(30);

    pub fn new(repo: PathBuf) -> Self {
        Self { repo }
    }

    pub fn command<Word: AsRef<OsStr>>(&self, words: impl IntoIterator<Item = Word>) -> Run {
        Run::git(&self.repo).args(words)
    }

    pub fn ran(run: Run) -> Result<Ran, LogFailure> {
        Ok(run.ran(Instant::now() + Self::LIMIT)?)
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::Git;
    use crate::git_log::content::Content;

    const NOWHERE: &str = "/nonexistent/meta";
    const STATUS: [&str; 1] = ["status"];
    const FAILED: &str = "git: git -C /nonexistent/meta status: ";

    #[test]
    fn failures_name_the_git_command() {
        let git = Git::new(PathBuf::from(NOWHERE));
        let failure =
            Content::try_from(Git::ran(git.command(STATUS)).unwrap()).err().map(|failure| failure.to_string());
        assert!(failure.is_some_and(|failure| failure.starts_with(FAILED)));
    }
}
