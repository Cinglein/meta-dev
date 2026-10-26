use std::path::{Path, PathBuf};

use meta_schema::{ByteCount, Removal, RemovedKind, Text};

use super::cargo_target::CargoTarget;
use super::footprint::Footprint;
use super::kept::Kept;
use super::work_tree::WorkTree;
use crate::request::cli_failure::Message;

#[derive(Debug)]
pub struct Vetted {
    path: PathBuf,
    kind: RemovedKind,
    bytes: ByteCount,
    place: Option<Text>,
}

impl Vetted {
    const GIT_ENTRY: &'static str = ".git";

    pub fn checked(given: &Path, project: &Path) -> Result<Self, Kept> {
        let entry = given.symlink_metadata().map_err(|_| Kept::NotFound)?;
        let path =
            (!entry.is_symlink()).then(|| given.canonicalize()).ok_or(Kept::Symlink)?.map_err(|_| Kept::NotFound)?;
        let current = project.canonicalize().map_err(|failure| Kept::Uninspected(Message::new(failure)))?;
        let inside = path.strip_prefix(&current).ok().and_then(|rest| rest.to_str()?.parse().ok());
        match (current.starts_with(&path), CargoTarget::tagged(&path), path.join(Self::GIT_ENTRY).exists()) {
            (true, _, _) => Err(Kept::HoldsCurrentRepo),
            (false, true, _) => CargoTarget::verified(&path).map(|bytes| Self {
                path,
                kind: RemovedKind::TargetDir,
                bytes,
                place: inside,
            }),
            (false, false, true) => Self::checkout(path, inside),
            (false, false, false) => Err(Kept::Unrecognised),
        }
    }

    fn checkout(path: PathBuf, inside: Option<Text>) -> Result<Self, Kept> {
        let locations = WorkTree::new(&path).settled()?;
        Ok(Self {
            kind: locations.kind(&path)?,
            bytes: Footprint::measured(&path),
            place: inside.or_else(|| locations.label()),
            path,
        })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn kind(&self) -> RemovedKind {
        self.kind
    }

    pub fn removal(&self) -> Removal {
        Removal::new(self.kind, self.bytes, self.place.clone())
    }
}
