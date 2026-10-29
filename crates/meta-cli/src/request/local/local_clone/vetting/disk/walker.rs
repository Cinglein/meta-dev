use std::fs::{DirEntry, ReadDir};
use std::os::unix::fs::MetadataExt as _;
use std::path::Path;

use crate::request::local::byte_count::ByteCount;
use crate::request::local::local_clone::vetting::entry_kind::EntryKind;
use crate::request::local::local_clone::vetting::found::Found;
use crate::request::local::local_clone::vetting::shared_node::{LinkCount, NodeId, SharedNode};

#[derive(Clone, Copy, Debug)]
pub struct Walker<'root>(&'root Path);

impl<'root> Walker<'root> {
    pub fn new(root: &'root Path) -> Self {
        Self(root)
    }

    pub fn descended(self, relative: &Path) -> Vec<Found> {
        self.0
            .join(relative)
            .read_dir()
            .map_or_else(|_| Self::unreadable(relative), |entries| self.listed(relative, entries))
    }

    fn listed(self, relative: &Path, entries: ReadDir) -> Vec<Found> {
        let entered = |entry: DirEntry| self.entered(relative, &entry);
        entries.flat_map(|entry| entry.map_or_else(|_| Self::unreadable(relative), entered)).collect()
    }

    fn entered(self, relative: &Path, entry: &DirEntry) -> Vec<Found> {
        let path = relative.join(entry.file_name());
        let found = |kind, bytes| Found::new(path.clone(), kind, bytes);
        match entry.path().symlink_metadata() {
            Ok(meta) if meta.is_dir() => {
                [vec![found(EntryKind::Dir, ByteCount::default())], self.descended(&path)].concat()
            }
            Ok(meta) if meta.is_symlink() => vec![found(EntryKind::Symlink, ByteCount::new(meta.len()))],
            Ok(meta) if meta.nlink() > 1 => {
                let shared = SharedNode::new(NodeId::new(meta.dev(), meta.ino()), LinkCount::new(meta.nlink()));
                vec![found(EntryKind::File, ByteCount::new(meta.len())).sharing(shared)]
            }
            Ok(meta) => vec![found(EntryKind::File, ByteCount::new(meta.len()))],
            Err(_) => Self::unreadable(&path),
        }
    }

    fn unreadable(path: &Path) -> Vec<Found> {
        vec![Found::new(path.to_path_buf(), EntryKind::Unreadable, ByteCount::default())]
    }
}
