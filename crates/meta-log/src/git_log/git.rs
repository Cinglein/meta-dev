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
    const LONG: Duration = Duration::from_secs(5);
    const SHORT: Duration = Duration::from_millis(5);

    #[test]
    fn failures_name_the_git_command_a_git_built_earlier_still_runs_and_rests_end_at_the_deadline() {
        let git = Git::new(PathBuf::from(NOWHERE), Instant::now() + SHORT);
        std::thread::sleep(SHORT);
        let failure = git.ran(git.command(STATUS)).map(|ran| Content::try_from(ran).err().map(|ran| ran.to_string()));
        assert!(failure.is_ok_and(|failure| failure.is_some_and(|failure| failure.starts_with(FAILED))));
        let started = Instant::now();
        Git::new(PathBuf::from(NOWHERE), started + SHORT).rest(LONG);
        assert!((SHORT..LONG).contains(&started.elapsed()));
    }
}
