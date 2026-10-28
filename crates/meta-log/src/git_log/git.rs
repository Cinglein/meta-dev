use std::ffi::OsStr;
use std::path::PathBuf;

use meta_run::{Budget, Ran, Run};

use crate::log_failure::LogFailure;

#[derive(Clone, Debug)]
pub struct Git {
    repo: PathBuf,
    budget: Budget,
}

impl Git {
    pub fn new(repo: PathBuf, budget: Budget) -> Self {
        Self { repo, budget }
    }

    pub fn command<Word: AsRef<OsStr>>(&self, words: impl IntoIterator<Item = Word>) -> Run {
        Run::git(&self.repo).args(words)
    }

    pub fn ran(&self, run: Run) -> Result<Ran, LogFailure> {
        Ok(run.ran(self.budget.deadline())?)
    }

    pub fn budget(&self) -> &Budget {
        &self.budget
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use std::time::Duration;

    use super::{Budget, Git};
    use crate::git_log::content::Content;

    const NOWHERE: &str = "/nonexistent/meta";
    const STATUS: [&str; 1] = ["status"];
    const FAILED: &str = "git: git -C /nonexistent/meta status: ";

    #[test]
    fn failures_name_the_git_command() {
        let git = Git::new(PathBuf::from(NOWHERE), Budget::new(Duration::from_secs(5)));
        let failure = git.ran(git.command(STATUS)).map(|ran| Content::try_from(ran).err().map(|ran| ran.to_string()));
        assert!(failure.is_ok_and(|failure| failure.is_some_and(|failure| failure.starts_with(FAILED))));
    }
}
