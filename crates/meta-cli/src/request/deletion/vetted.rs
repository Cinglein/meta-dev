use std::path::{Path, PathBuf};

use meta_schema::RemovedKind;

use super::cargo_tag::CargoTag;
use super::kept::Kept;
use super::work_tree::WorkTree;
use crate::request::cli_failure::Message;

#[derive(Debug)]
pub struct Vetted {
    path: PathBuf,
    kind: RemovedKind,
}

impl Vetted {
    const GIT_ENTRY: &'static str = ".git";

    pub fn checked(given: &Path, project: &Path) -> Result<Self, Kept> {
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
        Ok(Self { path, kind })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn kind(&self) -> RemovedKind {
        self.kind
    }
}
