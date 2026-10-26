use std::path::{Path, PathBuf};
use std::str::FromStr;

use meta_schema::{RemovedKind, Text};

use crate::request::deletion::kept::Kept;

#[derive(Debug)]
pub struct Locations {
    top: PathBuf,
    git_dir: PathBuf,
    common: PathBuf,
    branch: Text,
}

impl Locations {
    const LINKED: &'static str = "worktrees";
    const MODULES: &'static str = "modules";

    pub fn kind(&self, root: &Path) -> Result<RemovedKind, Kept> {
        match (
            self.top == root,
            self.common.join(Self::MODULES).exists(),
            self.git_dir == self.common,
            self.common.join(Self::LINKED).read_dir().is_ok_and(|mut entries| entries.next().is_some()),
        ) {
            (false, _, _, _) => Err(Kept::Unrecognised),
            (true, true, _, _) => Err(Kept::Submodules),
            (true, false, true, true) => Err(Kept::LinkedWorktrees),
            (true, false, true, false) => Ok(RemovedKind::Clone),
            (true, false, false, _) => Ok(RemovedKind::Worktree),
        }
    }

    pub fn label(&self) -> Option<Text> {
        let repo = self.common.parent()?.file_name()?.to_str()?;
        [repo, "@", &self.branch.to_string()].concat().parse().ok()
    }
}

impl FromStr for Locations {
    type Err = Kept;

    fn from_str(reply: &str) -> Result<Self, Self::Err> {
        match reply.lines().collect::<Vec<_>>().as_slice() {
            [top, git_dir, common, branch] => Ok(Self {
                top: PathBuf::from(top),
                git_dir: PathBuf::from(git_dir),
                common: PathBuf::from(common),
                branch: branch.parse().map_err(|_| Kept::Unrecognised)?,
            }),
            _ => Err(Kept::Unrecognised),
        }
    }
}
