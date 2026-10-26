mod footprint;

use std::path::{Path, PathBuf};

use meta_schema::{ByteCount, Removal, RemovedKind};

use super::kept::Kept;
use super::occupancy::Occupancy;
use super::vetted::Vetted;
use super::work_tree::WorkTree;
use crate::request::cli_failure::Message;
use footprint::Footprint;

#[derive(Debug)]
pub struct Deletable {
    path: PathBuf,
    kind: RemovedKind,
    bytes: ByteCount,
}

impl Deletable {
    const HOME: &'static str = "~";

    pub fn vacated(vetted: &Vetted, occupancy: &Occupancy) -> Result<Self, Kept> {
        occupancy.vacant(vetted.path())?;
        Ok(Self { path: vetted.path().to_path_buf(), kind: vetted.kind(), bytes: Footprint::measured(vetted.path()) })
    }

    pub fn delete(self) -> Result<Removal, Kept> {
        let shown = std::env::var_os("HOME")
            .and_then(|home| {
                self.path
                    .strip_prefix(Path::new(&home).canonicalize().ok()?)
                    .ok()
                    .map(|rest| Path::new(Self::HOME).join(rest))
            })
            .unwrap_or_else(|| self.path.clone());
        let logged = shown.to_string_lossy().parse().map_err(|failure| Kept::Uninspected(Message::new(failure)))?;
        match self.kind {
            RemovedKind::Worktree => WorkTree::new(&self.path).unlink(),
            RemovedKind::TargetDir | RemovedKind::Clone => {
                std::fs::remove_dir_all(&self.path).map_err(|failure| Kept::Unremoved(Message::new(failure)))
            }
        }?;
        Ok(Removal::new(logged, self.kind, self.bytes))
    }
}
