use std::ffi::OsStr;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use meta_run::{Ran, Run};

use crate::log_failure::LogFailure;

#[derive(Clone, Debug)]
pub struct Git {
    repo: PathBuf,
    deadline: Instant,
}

impl Git {
    pub fn new(repo: PathBuf, deadline: Instant) -> Self {
        Self { repo, deadline }
    }

    pub fn command<Word: AsRef<OsStr>>(&self, words: impl IntoIterator<Item = Word>) -> Run {
        Run::git(&self.repo).args(words)
    }

    pub fn ran(&self, run: Run) -> Result<Ran, LogFailure> {
        Ok(run.ran(self.deadline)?)
    }

    pub fn rest(&self, pause: Duration) {
        std::thread::sleep(pause.min(self.deadline.saturating_duration_since(Instant::now())));
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use std::time::{Duration, Instant};

    use super::Git;
    use crate::git_log::content::Content;

    const NOWHERE: &str = "/nonexistent/meta";
    const STATUS: [&str; 1] = ["status"];
    const FAILED: &str = "git: git -C /nonexistent/meta status: ";
    const PATIENT: Duration = Duration::from_secs(10);
    const LONG: Duration = Duration::from_secs(5);

    #[test]
    fn failures_name_the_git_command_and_rests_end_at_the_deadline() {
        let git = Git::new(PathBuf::from(NOWHERE), Instant::now() + PATIENT);
        let failure = Content::try_from(git.ran(git.command(STATUS)).unwrap()).err().map(|failure| failure.to_string());
        assert!(failure.is_some_and(|failure| failure.starts_with(FAILED)));
        let started = Instant::now();
        Git::new(PathBuf::from(NOWHERE), started).rest(LONG);
        assert!(started.elapsed() < LONG);
    }
}
