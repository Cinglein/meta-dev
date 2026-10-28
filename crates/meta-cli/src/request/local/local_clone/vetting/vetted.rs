mod checkouts;
mod index;
mod link;
mod porcelain;
mod ref_line;
mod ref_list;
mod tree;

use std::ffi::OsStr;
use std::path::Path;

use super::context::Context;
use super::entry_kind::EntryKind;
use super::inspection::Inspection;
use super::kept::Kept;
use crate::request::local::byte_count::ByteCount;
use crate::request::local::recorded_worktree::RecordedWorktree;
pub use checkouts::Checkouts;
use index::Index;
use link::Scope;
pub use link::{Link, Step};
use porcelain::Porcelain;
use ref_line::RefLine;
use ref_list::RefList;
use tree::Tree;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Vetted {
    link: Link,
    refs: RefList,
    branch: RefLine,
    bytes: ByteCount,
}

impl Vetted {
    const INDEX: [&'static str; 4] = ["ls-files", "-s", "-v", "-z"];
    const STATUS: [&'static str; 7] = [
        "status",
        "--porcelain",
        "-z",
        "--untracked-files=all",
        "--ignored=matching",
        "--ignore-submodules=none",
        "--no-renames",
    ];
    const REMOTE: &'static str = "ls-remote";
    const BEYOND: [&'static str; 3] = ["rev-list", "--max-count=1", "--ignore-missing"];
    const NOT: &'static str = "--not";
    const MODULES: &'static str = ".gitmodules";
    const TAG: &'static str = "CACHEDIR.TAG";

    pub fn proved(worktree: &RecordedWorktree, context: &Context, host: &impl Inspection) -> Result<Self, Kept> {
        Self::judged(worktree, context, host, |link, branch| Checkouts::held_now(link, branch, host))
    }

    pub fn proved_among(
        worktree: &RecordedWorktree,
        context: &Context,
        host: &impl Inspection,
        checkouts: &Result<Checkouts, Kept>,
    ) -> Result<Self, Kept> {
        Self::judged(worktree, context, host, |link, branch| {
            checkouts.as_ref().map_err(Kept::clone)?.held(branch, link.path())
        })
    }

    fn judged<Host: Inspection>(
        worktree: &RecordedWorktree,
        context: &Context,
        host: &Host,
        held: impl FnOnce(&Link, &str) -> Result<(), Kept>,
    ) -> Result<Self, Kept> {
        let path = worktree.path();
        Self::placed(path, context, host)?;
        let link = Link::proved(path, worktree.worktree(), context, host)?;
        let branch = worktree.branch().qualified();
        held(&link, &branch)?;
        let refs = Self::pushed(&link, &branch, context, host)?;
        let bytes = Self::settled(&link, host)?;
        Ok(Self { branch: refs.named(&branch).cloned().ok_or(Kept::NoBranch)?, refs, link, bytes })
    }

    pub fn rechecked(&self, held: &Link, host: &impl Inspection) -> Result<(), Kept> {
        (Self::settled(held, host)? == self.bytes && held.refs(host, self.branch.name())? == self.refs)
            .then_some(())
            .ok_or(Kept::Changed)
    }

    pub fn link(&self) -> &Link {
        &self.link
    }

    pub fn branch(&self) -> &RefLine {
        &self.branch
    }

    pub fn bytes(&self) -> ByteCount {
        self.bytes
    }

    fn placed(path: &Path, context: &Context, host: &impl Inspection) -> Result<(), Kept> {
        match host.entry(path) {
            EntryKind::Dir => Ok(()),
            EntryKind::Symlink => Err(Kept::Symlink),
            EntryKind::Missing => Err(Kept::NotFound),
            EntryKind::Unreadable | EntryKind::File => Err(Kept::Unreadable),
        }?;
        let canonical = host.canonical(path).filter(|canonical| canonical == path);
        canonical.and_then(|path| context.enclosed(&path)).ok_or(Kept::Escaped)?;
        let lent = context.lenders(host).iter().find_map(|alternates| alternates.lends_into(path));
        lent.map_or(Ok(()), |()| Err(Kept::Borrowed))
    }

    fn settled(link: &Link, host: &impl Inspection) -> Result<ByteCount, Kept> {
        let path = link.path();
        let tree = Tree::new(host.walked(path));
        let index: Index = link.git(host, Scope::Worktree, &Self::INDEX.map(OsStr::new))?.parse()?;
        tree.settled(index.links())?;
        (host.entry(&path.join(Self::MODULES)) == EntryKind::Missing).then_some(()).ok_or(Kept::Submodules)?;
        let Ok(status) = link.git(host, Scope::Worktree, &Self::STATUS.map(OsStr::new))?.parse::<Porcelain>();
        status.ignored()?.iter().try_for_each(|ignored| {
            tree.cargo_target(ignored, host.text(&path.join(ignored).join(Self::TAG)).as_deref())
        })?;
        Ok(tree.bytes())
    }

    fn pushed(link: &Link, branch: &str, context: &Context, host: &impl Inspection) -> Result<RefList, Kept> {
        let local = link.refs(host, branch)?;
        let listed = link.git(host, Scope::Common, &[OsStr::new(Self::REMOTE), context.remote().as_ref()]);
        let remote: RefList = listed.map_err(|kept| kept.recast(Kept::Unreachable))?.parse()?;
        local.matched(&remote)?;
        let beyond = Self::BEYOND.map(OsStr::new).into_iter().chain(local.objects()).chain([OsStr::new(Self::NOT)]);
        let words: Vec<&OsStr> = beyond.chain(remote.objects()).collect();
        link.git(host, Scope::Common, &words)?.lines().next().is_none().then_some(()).ok_or(Kept::Unpushed)?;
        (link.refs(host, branch)? == local).then_some(local).ok_or(Kept::Changed)
    }
}
