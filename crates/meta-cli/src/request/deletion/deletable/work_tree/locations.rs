use std::path::{Path, PathBuf};
use std::str::FromStr;

use meta_schema::RemovedKind;

use crate::request::deletion::kept::Kept;

#[derive(Debug)]
pub struct Locations {
    top: PathBuf,
    git_dir: PathBuf,
    common: PathBuf,
}

impl Locations {
    const LINKED: &'static str = "worktrees";

    pub fn kind(&self, root: &Path) -> Result<RemovedKind, Kept> {
        match (
            self.top == root,
            self.git_dir == self.common,
            self.common.join(Self::LINKED).read_dir().is_ok_and(|mut entries| entries.next().is_some()),
        ) {
            (false, _, _) => Err(Kept::Unrecognised),
            (true, true, true) => Err(Kept::LinkedWorktrees),
            (true, true, false) => Ok(RemovedKind::Clone),
            (true, false, _) => Ok(RemovedKind::Worktree),
        }
    }
}

impl FromStr for Locations {
    type Err = Kept;

    fn from_str(reply: &str) -> Result<Self, Self::Err> {
        match reply.lines().map(PathBuf::from).collect::<Vec<_>>().as_slice() {
            [top, git_dir, common] => Ok(Self { top: top.clone(), git_dir: git_dir.clone(), common: common.clone() }),
            _ => Err(Kept::Unrecognised),
        }
    }
}
