use std::path::{Path, PathBuf};

use meta_schema::RemovedKind;

use crate::request::cli_failure::Message;
use crate::request::deletion::kept::Kept;
use crate::request::deletion::occupancy::Occupancy;
use crate::request::deletion::vetted::Vetted;
use crate::request::deletion::work_tree::WorkTree;

#[derive(Debug)]
pub struct Deletable {
    path: PathBuf,
    kind: RemovedKind,
}

impl Deletable {
    pub fn confirmed(vetted: &Vetted, occupancy: &Occupancy, project: &Path) -> Result<Self, Kept> {
        occupancy.vacant(vetted.path())?;
        let again = Vetted::checked(vetted.path(), project)?;
        (again.kind() == vetted.kind())
            .then(|| Self { path: again.path().to_path_buf(), kind: again.kind() })
            .ok_or(Kept::Changed)
    }

    pub fn delete(self) -> Result<(), Kept> {
        match self.kind {
            RemovedKind::Worktree => WorkTree::new(&self.path).unlink(),
            RemovedKind::TargetDir | RemovedKind::Clone => {
                std::fs::remove_dir_all(&self.path).map_err(|failure| Kept::Unremoved(Message::new(failure)))
            }
        }
    }
}
