use std::path::{Path, PathBuf};

use super::Link;
use super::path_line::PathLine;
use crate::request::local::local_clone::vetting::disk::Disk;
use crate::request::local::local_clone::vetting::entry_kind::EntryKind;
use crate::request::local::local_clone::vetting::kept::Kept;
use crate::request::local::worktree_id::WorktreeId;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Pointers {
    entry: EntryKind,
    git_dir: Option<PathBuf>,
    back: Option<PathBuf>,
    shared: Option<PathBuf>,
}

impl Pointers {
    const ENTRY: &'static str = ".git";
    const LINKED: &'static str = "worktrees";
    const BACK: &'static str = "gitdir";
    const COMMON: &'static str = "commondir";

    pub fn observed(path: &Path) -> Self {
        let entry = path.join(Self::ENTRY);
        let git_dir = Self::followed(&entry, path);
        let from = |name: &str| git_dir.as_ref().and_then(|git_dir| Self::followed(&git_dir.join(name), git_dir));
        Self { entry: Disk::entry(&entry), back: from(Self::BACK), shared: from(Self::COMMON), git_dir }
    }

    #[cfg(test)]
    pub fn new(entry: EntryKind, git_dir: Option<PathBuf>, back: Option<PathBuf>, shared: Option<PathBuf>) -> Self {
        Self { entry, git_dir, back, shared }
    }

    pub fn proved(&self, path: &Path, id: Option<&WorktreeId>, common: &Path) -> Result<Link, Kept> {
        (self.entry == EntryKind::File).then_some(()).ok_or(Kept::Unlinked)?;
        let git_dir = self
            .git_dir
            .clone()
            .filter(|git_dir| git_dir.parent() == Some(common.join(Self::LINKED).as_path()))
            .ok_or(Kept::Unlinked)?;
        id.ok_or(Kept::Unidentified)?.names(&git_dir).then_some(()).ok_or(Kept::Stranger)?;
        (self.back.as_deref() == Some(path.join(Self::ENTRY).as_path())).then_some(()).ok_or(Kept::Escaped)?;
        (self.shared.as_deref() == Some(common)).then_some(()).ok_or(Kept::Escaped)?;
        Ok(Link::new(path, git_dir, common))
    }

    fn followed(file: &Path, base: &Path) -> Option<PathBuf> {
        let Ok(line) = Disk::text(file)?.parse::<PathLine>();
        Disk::canonical(&line.resolved(base))
    }
}
