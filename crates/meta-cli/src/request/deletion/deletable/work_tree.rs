mod locations;
mod porcelain;
mod revisions;

use std::path::{Path, PathBuf};
use std::process::Command;

use meta_schema::RemovedKind;

use crate::request::cli_failure::Message;
use crate::request::deletion::kept::Kept;
use locations::Locations;
use porcelain::Porcelain;
use revisions::Revisions;

#[derive(Debug)]
pub struct WorkTree {
    root: PathBuf,
}

impl WorkTree {
    const LOCATIONS: [&'static str; 5] =
        ["rev-parse", "--path-format=absolute", "--show-toplevel", "--git-dir", "--git-common-dir"];
    const STATUS: [&'static str; 4] = ["status", "--porcelain", "-z", "--ignored"];
    const UNPUSHED: [&'static str; 7] =
        ["rev-list", "--max-count=1", "HEAD", "--branches", "--glob=stash", "--not", "--remotes"];
    const REMOVE: [&'static str; 3] = ["worktree", "remove", "."];

    pub fn new(root: &Path) -> Self {
        Self { root: root.to_path_buf() }
    }

    pub fn settled_kind(&self) -> Result<RemovedKind, Kept> {
        let kind = self.run(&Self::LOCATIONS)?.parse::<Locations>()?.kind(&self.root)?;
        self.run(&Self::STATUS)?.parse::<Porcelain>()?.settled(&self.root)?;
        self.run(&Self::UNPUSHED)?.parse::<Revisions>()?;
        Ok(kind)
    }

    pub fn unlink(self) -> Result<(), Kept> {
        self.run(&Self::REMOVE).map(drop).map_err(|kept| Kept::Unremoved(Message::new(kept)))
    }

    fn run(&self, words: &[&str]) -> Result<String, Kept> {
        let ran = Command::new("git")
            .arg("-C")
            .arg(&self.root)
            .args(words)
            .output()
            .map_err(|failure| Kept::Uninspected(Message::new(failure)))?;
        ran.status
            .success()
            .then(|| String::from_utf8_lossy(&ran.stdout).into_owned())
            .ok_or_else(|| Kept::Uninspected(Message::new(String::from_utf8_lossy(&ran.stderr))))
    }
}
