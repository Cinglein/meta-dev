mod git_command;

use std::ffi::OsStr;
use std::path::PathBuf;
use std::process::Command;

pub use git_command::GitCommand;

#[derive(Clone, Debug)]
pub struct Git {
    repo: PathBuf,
}

impl Git {
    const ISOLATED: [&'static str; 3] = ["GIT_DIR", "GIT_WORK_TREE", "GIT_INDEX_FILE"];

    pub fn new(repo: PathBuf) -> Self {
        Self { repo }
    }

    pub fn command<Word: AsRef<OsStr>>(&self, words: impl IntoIterator<Item = Word>) -> GitCommand {
        let mut command = Command::new("git");
        command.arg("-C").arg(&self.repo).args(words);
        Self::ISOLATED.iter().fold(&mut command, |command, variable| command.env_remove(variable));
        GitCommand::new(command)
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::Git;

    const NOWHERE: &str = "/nonexistent/meta";
    const STATUS: [&str; 1] = ["status"];
    const FAILED: &str = "git: git -C /nonexistent/meta status: ";

    #[test]
    fn failures_name_the_git_command() {
        let failure =
            Git::new(PathBuf::from(NOWHERE)).command(STATUS).content().err().map(|failure| failure.to_string());
        assert!(failure.is_some_and(|failure| failure.starts_with(FAILED)));
    }
}
