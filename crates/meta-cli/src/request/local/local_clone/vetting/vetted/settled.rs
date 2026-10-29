use std::path::{Path, PathBuf};

use meta_run::Budget;

use super::index::Index;
use super::link::{Link, Scope};
use super::porcelain::Porcelain;
use super::tree::Tree;
use crate::request::local::byte_count::ByteCount;
use crate::request::local::local_clone::vetting::asked::{IndexEntries, StatusEntries};
use crate::request::local::local_clone::vetting::disk::Disk;
use crate::request::local::local_clone::vetting::entry_kind::EntryKind;
use crate::request::local::local_clone::vetting::kept::Kept;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Settled {
    tree: Tree,
    index: Result<Index, Kept>,
    modules: EntryKind,
    status: Result<Porcelain, Kept>,
    tags: Vec<(PathBuf, Option<String>)>,
}

impl Settled {
    const MODULES: &'static str = ".gitmodules";
    const TAG: &'static str = "CACHEDIR.TAG";

    pub fn observed(link: &Link, budget: Budget) -> Self {
        let path = link.path();
        let repo = link.repo(Scope::Worktree, budget);
        let tree = Tree::new(Disk::walked(path));
        let index = repo.observe(&IndexEntries);
        let modules = Disk::entry(&path.join(Self::MODULES));
        let clean = index.as_ref().map_err(Kept::clone).and_then(|index| tree.settled(index.links()));
        let unmoduled = clean.and_then(|()| (modules == EntryKind::Missing).then_some(()).ok_or(Kept::Submodules));
        let status = unmoduled.and_then(|()| repo.observe(&StatusEntries));
        let ignored = status.as_ref().ok().and_then(|status| status.ignored().ok()).unwrap_or_default();
        let tags = ignored.iter().map(|dir| (dir.to_path_buf(), Disk::text(&path.join(dir).join(Self::TAG)))).collect();
        Self { tree, index, modules, status, tags }
    }

    #[cfg(test)]
    pub fn new(
        tree: Tree,
        index: Result<Index, Kept>,
        modules: EntryKind,
        status: Result<Porcelain, Kept>,
        tags: Vec<(PathBuf, Option<String>)>,
    ) -> Self {
        Self { tree, index, modules, status, tags }
    }

    pub fn judged(&self) -> Result<ByteCount, Kept> {
        let index = self.index.clone()?;
        self.tree.settled(index.links())?;
        (self.modules == EntryKind::Missing).then_some(()).ok_or(Kept::Submodules)?;
        let status = self.status.clone()?;
        status.ignored()?.iter().try_for_each(|ignored| self.tree.cargo_target(ignored, self.tag(ignored)))?;
        Ok(self.tree.bytes())
    }

    fn tag(&self, dir: &Path) -> Option<&str> {
        self.tags.iter().find(|(tagged, _)| tagged == dir).and_then(|(_, tag)| tag.as_deref())
    }
}
