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
use index::Index;
use link::Link;
use porcelain::Porcelain;
use ref_line::RefLine;
use ref_list::RefList;
use tree::Tree;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Vetted {
    link: Link,
    refs: RefList,
    branch: Option<RefLine>,
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
    const HEAD: [&'static str; 3] = ["rev-parse", "--verify", "HEAD"];
    const REFS: [&'static str; 5] =
        ["for-each-ref", "--format=%(objectname) %(refname)", "refs/worktree/", "refs/bisect/", "refs/rewritten/"];
    const REMOTE: &'static str = "ls-remote";
    const BEYOND: [&'static str; 3] = ["rev-list", "--max-count=1", "--ignore-missing"];
    const NOT: &'static str = "--not";
    const MODULES: &'static str = ".gitmodules";
    const TAG: &'static str = "CACHEDIR.TAG";

    pub fn proved(worktree: &RecordedWorktree, context: &Context, host: &impl Inspection) -> Result<Self, Kept> {
        let path = worktree.path();
        Self::placed(path, context, host)?;
        let link = Link::proved(path, context, host)?;
        let bytes = Self::settled(&link, host)?;
        let branch = worktree.branch().qualified();
        let refs = Self::pushed(&link, &branch, context, host)?;
        Ok(Self { branch: refs.named(&branch).cloned(), refs, link, bytes })
    }

    pub fn link(&self) -> &Link {
        &self.link
    }

    pub fn branch(&self) -> Option<&RefLine> {
        self.branch.as_ref()
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
        let index: Index = link.git(host, &Self::INDEX.map(OsStr::new))?.parse()?;
        tree.settled(index.links())?;
        (host.entry(&path.join(Self::MODULES)) == EntryKind::Missing).then_some(()).ok_or(Kept::Submodules)?;
        let Ok(status) = link.git(host, &Self::STATUS.map(OsStr::new))?.parse::<Porcelain>();
        status.ignored()?.iter().try_for_each(|ignored| {
            tree.cargo_target(ignored, host.text(&path.join(ignored).join(Self::TAG)).as_deref())
        })?;
        Ok(tree.bytes())
    }

    fn pushed(link: &Link, branch: &str, context: &Context, host: &impl Inspection) -> Result<RefList, Kept> {
        let head = RefLine::head(&link.git(host, &Self::HEAD.map(OsStr::new))?);
        let named = Self::REFS.map(OsStr::new).into_iter().chain([OsStr::new(branch)]).collect::<Vec<_>>();
        let local = link.git(host, &named)?.parse::<RefList>()?.headed(head);
        let listed = link.git(host, &[OsStr::new(Self::REMOTE), context.remote().as_ref()]);
        let remote: RefList = listed.map_err(|kept| kept.recast(Kept::Unreachable))?.parse()?;
        local.matched(&remote)?;
        let beyond = Self::BEYOND.map(OsStr::new).into_iter().chain(local.objects()).chain([OsStr::new(Self::NOT)]);
        let words: Vec<&OsStr> = beyond.chain(remote.objects()).collect();
        link.git(host, &words)?.lines().next().is_none().then_some(local).ok_or(Kept::Unpushed)
    }
}
