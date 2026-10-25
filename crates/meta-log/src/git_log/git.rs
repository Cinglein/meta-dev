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
        Self::ISOLATED.iter().for_each(|variable| {
            command.env_remove(variable);
        });
        GitCommand::new(command)
    }
}
