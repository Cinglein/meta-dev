mod checkouts;
mod index;
mod link;
mod porcelain;
mod ref_line;
mod ref_list;
mod survey;
mod tree;

use std::ffi::OsStr;
use std::path::Path;

use super::context::Context;
use super::entry_kind::EntryKind;
use super::halt::Halt;
use super::kept::Kept;
use super::look::Look;
use super::replay::Replay;
use super::seen::Seen;
use crate::request::local::byte_count::ByteCount;
use crate::request::local::recorded_worktree::RecordedWorktree;
pub use checkouts::Checkouts;
use index::Index;
use link::Scope;
pub use link::{Link, Step};
use porcelain::Porcelain;
use ref_line::RefLine;
pub use ref_list::RefList;
use survey::Survey;
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
    const BEYOND: [&'static str; 3] = ["rev-list", "--max-count=1", "--ignore-missing"];
    const NOT: &'static str = "--not";
    const MODULES: &'static str = ".gitmodules";
    const TAG: &'static str = "CACHEDIR.TAG";

    pub fn proved(worktree: &RecordedWorktree, context: &Context, replay: &Replay<'_>) -> Result<Self, Halt> {
        Self::judged(worktree, context, replay, Survey::Fresh)
    }

    pub fn proved_among(
        worktree: &RecordedWorktree,
        context: &Context,
        replay: &Replay<'_>,
        checkouts: &Result<Checkouts, Kept>,
        remote: &Result<RefList, Kept>,
    ) -> Result<Self, Halt> {
        Self::judged(worktree, context, replay, Survey::Swept { checkouts, remote })
    }

    fn judged(
        worktree: &RecordedWorktree,
        context: &Context,
        replay: &Replay<'_>,
        survey: Survey<'_>,
    ) -> Result<Self, Halt> {
        let path = worktree.path();
        Self::placed(path, context, replay)?;
        let link = Link::proved(path, worktree.worktree(), context, replay)?;
        let branch = worktree.branch().qualified();
        survey.held(&link, &branch, replay)?;
        let refs = Self::pushed(&link, &branch, survey, context, replay)?;
        let bytes = Self::settled(&link, replay)?;
        Ok(Self { branch: refs.named(&branch).cloned().ok_or(Kept::NoBranch)?, refs, link, bytes })
    }

    // The live re-read: the tree and refs again, as new turns, compared with what the vet saw.
    pub fn rechecked(&self, held: &Link, replay: &Replay<'_>) -> Result<(), Halt> {
        (Self::settled(held, replay)? == self.bytes && held.refs(replay, self.branch.name())? == self.refs)
            .then_some(())
            .ok_or(Halt::Kept(Kept::Changed))
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

    fn placed(path: &Path, context: &Context, replay: &Replay<'_>) -> Result<(), Halt> {
        let [entry, canonical] = Self::batch(replay, [Look::Entry(path.to_path_buf()), Look::Canonical(path.to_path_buf())])?;
        match entry.entry()? {
            EntryKind::Dir => Ok(()),
            EntryKind::Symlink => Err(Kept::Symlink),
            EntryKind::Missing => Err(Kept::NotFound),
            EntryKind::Unreadable | EntryKind::File => Err(Kept::Unreadable),
        }?;
        let canonical = canonical.canonical()?.filter(|canonical| canonical == path);
        canonical.and_then(|path| context.enclosed(&path)).ok_or(Kept::Escaped)?;
        let lent = context.lenders().iter().find_map(|alternates| alternates.lends_into(path));
        Ok(lent.map_or(Ok(()), |()| Err(Kept::Borrowed))?)
    }

    fn settled(link: &Link, replay: &Replay<'_>) -> Result<ByteCount, Halt> {
        let path = link.path();
        let [walked, index, modules, status] = Self::batch(
            replay,
            [
                Look::Walk(path.to_path_buf()),
                Look::Git(link.call(Scope::Worktree, &Self::INDEX.map(OsStr::new))),
                Look::Entry(path.join(Self::MODULES)),
                Look::Git(link.call(Scope::Worktree, &Self::STATUS.map(OsStr::new))),
            ],
        )?;
        let tree = Tree::new(walked.walked()?);
        let index: Index = index.git()??.parse()?;
        tree.settled(index.links())?;
        (modules.entry()? == EntryKind::Missing).then_some(()).ok_or(Kept::Submodules)?;
        let Ok(status) = status.git()??.parse::<Porcelain>();
        let ignored = status.ignored()?;
        if !ignored.is_empty() {
            let tags = replay.looked(ignored.iter().map(|ignored| Look::Text(path.join(ignored).join(Self::TAG))).collect())?;
            ignored.iter().zip(tags).try_for_each(|(ignored, tag)| -> Result<(), Halt> {
                Ok(tree.cargo_target(ignored, tag.text()?.as_deref())?)
            })?;
        }
        Ok(tree.bytes())
    }

    fn pushed(
        link: &Link,
        branch: &str,
        survey: Survey<'_>,
        context: &Context,
        replay: &Replay<'_>,
    ) -> Result<RefList, Halt> {
        let local = link.refs(replay, branch)?;
        let remote = survey.remote_refs(context, replay)?;
        local.matched(&remote)?;
        let beyond = Self::BEYOND.map(OsStr::new).into_iter().chain(local.objects()).chain([OsStr::new(Self::NOT)]);
        let words: Vec<&OsStr> = beyond.chain(remote.objects()).collect();
        link.git(replay, Scope::Common, &words)??.lines().next().is_none().then_some(()).ok_or(Kept::Unpushed)?;
        (link.refs(replay, branch)? == local).then_some(local).ok_or(Halt::Kept(Kept::Changed))
    }

    fn batch<const N: usize>(replay: &Replay<'_>, looks: [Look; N]) -> Result<[Seen; N], Halt> {
        <[Seen; N]>::try_from(replay.looked(looks.to_vec())?).map_err(|_| Replay::diverged())
    }
}
