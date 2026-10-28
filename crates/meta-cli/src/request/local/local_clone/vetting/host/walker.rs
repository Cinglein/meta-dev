use meta_door::kinds::{AdminDir, Kind, Metadata, MetadataExt as _, Walked};
use meta_door::{Door, Entry};
use std::io;
use std::path::{Path, PathBuf};

use crate::request::local::byte_count::ByteCount;
use crate::request::local::local_clone::vetting::entry_kind::EntryKind;
use crate::request::local::local_clone::vetting::found::Found;
use crate::request::local::local_clone::vetting::shared_node::{LinkCount, NodeId, SharedNode};
use crate::request::local::local_clone::vetting::thaw::Thaw;

#[derive(Clone, Copy, Debug)]
pub struct Walker<'root>(&'root Path);

impl<'root> Walker<'root> {
    pub fn new(root: &'root Path) -> Self {
        Self(root)
    }

    pub fn inspected(path: &Path) -> io::Result<Metadata> {
        Door::described(Thaw::stored(path).and_then(Result::ok).unwrap_or_else(|| Entry::from(path)))
    }

    pub fn walked(self) -> Vec<Found> {
        Door::walked(self.0).map_or_else(
            |_| self.admin(Path::new("")),
            |walked| walked.iter().flat_map(|walked| self.found(walked)).collect(),
        )
    }

    fn found(self, walked: &Walked) -> Vec<Found> {
        let relative = walked.relative();
        let dir = || Found::new(relative.to_path_buf(), EntryKind::Dir, ByteCount::default());
        match (walked.kind(), walked.described()) {
            (Kind::Admin, _) => [vec![dir()], self.admin(relative)].concat(),
            (_, Some(meta)) => vec![Self::classified(relative, meta)],
            (_, None) => vec![Self::unreadable(relative)],
        }
    }

    fn admin(self, relative: &Path) -> Vec<Found> {
        let admin = Door::inspected(&self.0.join(relative));
        let inner = |admin: &AdminDir, path: &PathBuf| relative.join(path.strip_prefix(admin.path()).unwrap_or(path));
        let found = |admin: &AdminDir, path: &PathBuf| {
            let inner = inner(admin, path);
            admin.described(path).map_or_else(|_| Self::unreadable(&inner), |meta| Self::classified(&inner, &meta))
        };
        let listed = admin.and_then(|admin| Ok(admin.listed()?.iter().map(|path| found(&admin, path)).collect()));
        listed.unwrap_or_else(|_: std::io::Error| vec![Self::unreadable(relative)])
    }

    fn classified(path: &Path, meta: &Metadata) -> Found {
        let found = |kind, bytes| Found::new(path.to_path_buf(), kind, bytes);
        match Kind::classified(meta) {
            Kind::Dir => found(EntryKind::Dir, ByteCount::default()),
            Kind::Link => found(EntryKind::Symlink, ByteCount::new(meta.len())),
            _ if meta.nlink() > 1 => {
                let shared = SharedNode::new(NodeId::new(meta.dev(), meta.ino()), LinkCount::new(meta.nlink()));
                found(EntryKind::File, ByteCount::new(meta.len())).sharing(shared)
            }
            _ => found(EntryKind::File, ByteCount::new(meta.len())),
        }
    }

    fn unreadable(path: &Path) -> Found {
        Found::new(PathBuf::from(path), EntryKind::Unreadable, ByteCount::default())
    }
}
