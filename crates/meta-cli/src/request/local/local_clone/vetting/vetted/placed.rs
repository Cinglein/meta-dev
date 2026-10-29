use std::path::{Path, PathBuf};

use crate::request::local::local_clone::vetting::context::Context;
use crate::request::local::local_clone::vetting::disk::Disk;
use crate::request::local::local_clone::vetting::entry_kind::EntryKind;
use crate::request::local::local_clone::vetting::kept::Kept;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Placed {
    kind: EntryKind,
    canonical: Option<PathBuf>,
}

impl Placed {
    pub fn observed(path: &Path) -> Self {
        Self { kind: Disk::entry(path), canonical: Disk::canonical(path) }
    }

    #[cfg(test)]
    pub fn new(kind: EntryKind, canonical: Option<PathBuf>) -> Self {
        Self { kind, canonical }
    }

    pub fn judged(&self, path: &Path, context: &Context) -> Result<(), Kept> {
        match self.kind {
            EntryKind::Dir => Ok(()),
            EntryKind::Symlink => Err(Kept::Symlink),
            EntryKind::Missing => Err(Kept::NotFound),
            EntryKind::Unreadable | EntryKind::File => Err(Kept::Unreadable),
        }?;
        let canonical = self.canonical.as_deref().filter(|canonical| *canonical == path);
        canonical.and_then(|path| context.enclosed(path)).ok_or(Kept::Escaped)?;
        let lent = context.lenders().iter().find_map(|alternates| alternates.lends_into(path));
        lent.map_or(Ok(()), |()| Err(Kept::Borrowed))
    }
}
