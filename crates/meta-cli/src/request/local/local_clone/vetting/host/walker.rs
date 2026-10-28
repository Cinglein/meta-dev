use meta_door::kinds::{AdminDir, Kind, Metadata, MetadataExt as _};
use meta_door::{Door, Entry};
use std::io::{self, ErrorKind};
use std::path::{Path, PathBuf};

use crate::request::local::byte_count::ByteCount;
use crate::request::local::local_clone::vetting::entry_kind::EntryKind;
use crate::request::local::local_clone::vetting::found::Found;
use crate::request::local::local_clone::vetting::shared_node::{LinkCount, NodeId, SharedNode};
use crate::request::local::local_clone::vetting::thaw::Thaw;

#[derive(Clone, Debug)]
pub struct Walker<'root> {
    root: &'root Path,
    admin: Option<AdminDir>,
}

impl<'root> Walker<'root> {
    pub fn new(root: &'root Path) -> Self {
        Self { root, admin: None }
    }

    pub fn inspected(path: &Path) -> io::Result<Metadata> {
        let entry = Thaw::stored(path).and_then(Result::ok).unwrap_or_else(|| Entry::from(path));
        Door::described(entry).or_else(|failure| {
            let admin = Door::enclosing(path).filter(|_| failure.kind() != ErrorKind::NotFound);
            admin.ok_or(failure)?.described(path)
        })
    }

    pub fn descended(&self, relative: &Path) -> Vec<Found> {
        let directory = self.root.join(relative);
        let listed = self.admin.as_ref().map_or_else(|| Door::listed(&directory), |admin| admin.listed(&directory));
        match (listed, &self.admin) {
            (Ok(paths), _) => paths.iter().flat_map(|path| self.entered(relative, path)).collect(),
            (Err(_), Some(_)) => Self::unreadable(relative),
            (Err(_), None) => Door::enclosing(&directory).map_or_else(
                || Self::unreadable(relative),
                |admin| Self { root: self.root, admin: Some(admin) }.descended(relative),
            ),
        }
    }

    fn entered(&self, relative: &Path, entry: &Path) -> Vec<Found> {
        let path = relative.join(entry.file_name().unwrap_or_default());
        self.described(entry).map_or_else(|_| Self::unreadable(&path), |meta| self.classified(&path, &meta))
    }

    fn classified(&self, path: &Path, meta: &Metadata) -> Vec<Found> {
        let found = |kind, bytes| Found::new(path.to_path_buf(), kind, bytes);
        match Kind::classified(meta) {
            Kind::Dir => [vec![found(EntryKind::Dir, ByteCount::default())], self.descended(path)].concat(),
            Kind::Link => vec![found(EntryKind::Symlink, ByteCount::new(meta.len()))],
            _ if meta.nlink() > 1 => {
                let shared = SharedNode::new(NodeId::new(meta.dev(), meta.ino()), LinkCount::new(meta.nlink()));
                vec![found(EntryKind::File, ByteCount::new(meta.len())).sharing(shared)]
            }
            _ => vec![found(EntryKind::File, ByteCount::new(meta.len()))],
        }
    }

    fn described(&self, entry: &Path) -> io::Result<Metadata> {
        self.admin.as_ref().map_or_else(|| Self::inspected(entry), |admin| admin.described(entry))
    }

    fn unreadable(path: &Path) -> Vec<Found> {
        vec![Found::new(PathBuf::from(path), EntryKind::Unreadable, ByteCount::default())]
    }
}
