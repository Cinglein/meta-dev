mod cargo_tag;
mod footprint;
mod occupancy;
mod work_tree;

use std::path::{Path, PathBuf};

use meta_schema::{ByteCount, Removal, RemovedKind};

use super::kept::Kept;
use crate::request::cli_failure::Message;
use cargo_tag::CargoTag;
use footprint::Footprint;
use occupancy::Occupancy;
use work_tree::WorkTree;

#[derive(Debug)]
pub struct Deletable {
    path: PathBuf,
    kind: RemovedKind,
    bytes: ByteCount,
}

impl Deletable {
    const GIT_ENTRY: &'static str = ".git";
    const HOME: &'static str = "~";

    pub fn vetted(given: &Path, project: &Path) -> Result<Self, Kept> {
        let entry = given.symlink_metadata().map_err(|_| Kept::NotFound)?;
        let path =
            (!entry.is_symlink()).then(|| given.canonicalize()).ok_or(Kept::Symlink)?.map_err(|_| Kept::NotFound)?;
        let current = project.canonicalize().map_err(|failure| Kept::Uninspected(Message::new(failure)))?;
        let kind = match (current.starts_with(&path), CargoTag::marks(&path), path.join(Self::GIT_ENTRY).exists()) {
            (true, _, _) => Err(Kept::HoldsCurrentRepo),
            (false, true, _) => Ok(RemovedKind::TargetDir),
            (false, false, true) => WorkTree::new(&path).settled_kind(),
            (false, false, false) => Err(Kept::Unrecognised),
        }?;
        Occupancy::vacant(&path)?;
        Ok(Self { bytes: Footprint::measured(&path), path, kind })
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
