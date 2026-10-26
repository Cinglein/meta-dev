mod borrowers;
mod index;
mod locations;
mod porcelain;
mod revisions;

use std::path::{Path, PathBuf};
use std::process::Command;

use super::kept::Kept;
use crate::request::cli_failure::Message;
use borrowers::Borrowers;
use index::Index;
use locations::Locations;
use porcelain::Porcelain;
use revisions::Revisions;

#[derive(Debug)]
pub struct WorkTree {
    root: PathBuf,
}

impl WorkTree {
    const LOCATIONS: [&'static str; 7] = [
        "rev-parse",
        "--path-format=absolute",
        "--show-toplevel",
        "--git-dir",
        "--git-common-dir",
        "--abbrev-ref",
        "HEAD",
    ];
    const INDEX: [&'static str; 4] = ["ls-files", "-v", "-s", "-z"];
    const STATUS: [&'static str; 9] = [
        "-c",
        "core.fsmonitor=false",
        "status",
        "--porcelain",
        "-z",
        "--untracked-files=all",
        "--ignored=matching",
        "--ignore-submodules=none",
        "--no-renames",
    ];
    const UNPUSHED: [&'static str; 5] = ["rev-list", "--max-count=1", "--all", "--not", "--remotes"];
    const REMOVE: [&'static str; 3] = ["worktree", "remove", "."];

    pub fn new(root: &Path) -> Self {
        Self { root: root.to_path_buf() }
    }

    pub fn settled(&self) -> Result<Locations, Kept> {
        let locations: Locations = self.run(&Self::LOCATIONS)?.parse()?;
        Borrowers::none_of(&self.root, locations.kind(&self.root)?)?;
        self.run(&Self::INDEX)?.parse::<Index>()?;
        self.run(&Self::STATUS)?.parse::<Porcelain>()?.settled(&self.root)?;
        self.run(&Self::UNPUSHED)?.parse::<Revisions>()?;
        Ok(locations)
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
