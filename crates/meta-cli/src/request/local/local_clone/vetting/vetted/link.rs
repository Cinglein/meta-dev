mod path_line;
mod root_ref;
mod scope;
mod step;
mod worktree_config;

use std::ffi::OsStr;
use std::path::{Path, PathBuf};

use super::ref_line::RefLine;
use super::ref_list::RefList;
use crate::request::local::local_clone::vetting::context::Context;
use crate::request::local::local_clone::vetting::entry_kind::EntryKind;
use crate::request::local::local_clone::vetting::git_call::GitCall;
use crate::request::local::local_clone::vetting::inspection::Inspection;
use crate::request::local::local_clone::vetting::kept::Kept;
use crate::request::local::local_clone::vetting::thaw::Thaw;
use crate::request::local::worktree_id::WorktreeId;
use path_line::PathLine;
use root_ref::RootRef;
pub use scope::Scope;
pub use step::Step;
use worktree_config::WorktreeConfig;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Link {
    path: PathBuf,
    git_dir: PathBuf,
    common: PathBuf,
}

impl Link {
    const ENTRY: &'static str = ".git";
    const LINKED: &'static str = "worktrees";
    const BACK: &'static str = "gitdir";
    const COMMON: &'static str = "commondir";
    const LOCKED: &'static str = "locked";
    const ALTERNATES: &'static str = "objects/info/alternates";
    const MODULES: &'static str = "modules";
    const PAUSED_ENTRIES: [&'static str; 7] =
        ["rebase-merge", "rebase-apply", "sequencer", "MERGE_HEAD", "CHERRY_PICK_HEAD", "REVERT_HEAD", "REBASE_HEAD"];
    const CONFIG: &'static str = "config.worktree";
    const REFTABLE: &'static str = "reftable";
    const RESOLVE: [&'static str; 3] = ["rev-parse", "--verify", "-q"];
    const HEAD: [&'static str; 3] = ["rev-parse", "--verify", "HEAD"];
    const REFS: [&'static str; 5] =
        ["for-each-ref", "--format=%(objectname) %(refname)", "refs/worktree/", "refs/bisect/", "refs/rewritten/"];
    const MOVE: [&'static str; 2] = ["worktree", "move"];
    const UNBRANCH: [&'static str; 2] = ["update-ref", "-d"];

    pub fn proved(
        path: &Path,
        id: Option<&WorktreeId>,
        context: &Context,
        host: &impl Inspection,
    ) -> Result<Self, Kept> {
        let entry = path.join(Self::ENTRY);
        (host.entry(&entry) == EntryKind::File).then_some(()).ok_or(Kept::Unlinked)?;
        let common = context.common();
        let git_dir = Self::followed(host, &entry, path)
            .filter(|git_dir| git_dir.parent() == Some(common.join(Self::LINKED).as_path()))
            .ok_or(Kept::Unlinked)?;
        id.ok_or(Kept::Unidentified)?.names(&git_dir).then_some(()).ok_or(Kept::Stranger)?;
        (Self::followed(host, &git_dir.join(Self::BACK), &git_dir) == Some(entry))
            .then_some(())
            .ok_or(Kept::Escaped)?;
        let shared = Self::followed(host, &git_dir.join(Self::COMMON), &git_dir);
        (shared.as_deref() == Some(common)).then_some(()).ok_or(Kept::Escaped)?;
        let link = Self { path: path.to_path_buf(), git_dir, common: common.to_path_buf() };
        link.guarded(host).map(|()| link)
    }

    pub fn git(&self, host: &impl Inspection, scope: Scope, words: &[&OsStr]) -> Result<String, Kept> {
        host.git(&match scope {
            Scope::Worktree => GitCall::worktree(&self.git_dir, &self.path, words),
            Scope::Common => GitCall::common(&self.common, words),
        })
    }

    pub fn refs(&self, host: &impl Inspection, branch: &str) -> Result<RefList, Kept> {
        let head = RefLine::head(&self.git(host, Scope::Worktree, &Self::HEAD.map(OsStr::new))?);
        let named = Self::REFS.map(OsStr::new).into_iter().chain([OsStr::new(branch)]).collect::<Vec<_>>();
        let local = self.git(host, Scope::Worktree, &named)?.parse::<RefList>()?.headed(head);
        let listed = host.walked(&self.git_dir);
        let named = listed.iter().filter(|found| found.kind() == EntryKind::File);
        let roots = named
            .filter_map(|found| found.path().to_str()?.parse::<RootRef>().ok())
            .map(|root| root.peeled())
            .filter_map(|peeled| self.resolved(host, &peeled).map(|object| RefLine::pseudo(&peeled, &object)));
        Ok(roots.fold(local, RefList::headed))
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn relocated(&self, path: &Path) -> Self {
        Self { path: path.to_path_buf(), ..self.clone() }
    }

    pub fn moving(&self, to: &Path) -> GitCall {
        let words = Self::MOVE.map(OsStr::new).into_iter().chain([self.path.as_os_str(), to.as_os_str()]);
        GitCall::common(&self.common, &words.collect::<Vec<_>>())
    }

    pub fn step(&self, step: Step) -> GitCall {
        let words = step.words().iter().map(OsStr::new).chain([self.path.as_os_str()]).collect::<Vec<_>>();
        GitCall::common(&self.common, &words)
    }

    pub fn thaw_plan(&self, origin: &Path) -> Thaw {
        Thaw::planned(origin, Thaw::ledger_of(&self.path, &self.common))
    }

    pub fn unbranch(&self, branch: &RefLine) -> GitCall {
        let words = Self::UNBRANCH.map(OsStr::new).into_iter().chain(branch.deletion()).collect::<Vec<_>>();
        GitCall::common(&self.common, &words)
    }

    fn resolved(&self, host: &impl Inspection, name: &str) -> Option<String> {
        let words = Self::RESOLVE.map(OsStr::new).into_iter().chain([OsStr::new(name)]).collect::<Vec<_>>();
        self.git(host, Scope::Worktree, &words).ok().filter(|object| object.lines().next().is_some())
    }

    fn guarded(&self, host: &impl Inspection) -> Result<(), Kept> {
        let absent =
            |dirs: &[&Path], name: &str| dirs.iter().all(|dir| host.entry(&dir.join(name)) == EntryKind::Missing);
        let both = [self.git_dir.as_path(), self.common.as_path()];
        absent(&[&self.git_dir], Self::LOCKED).then_some(()).ok_or(Kept::Locked)?;
        absent(&[&self.git_dir], Self::REFTABLE).then_some(()).ok_or(Kept::Unlistable)?;
        absent(&both, Self::ALTERNATES).then_some(()).ok_or(Kept::Borrowed)?;
        absent(&both, Self::MODULES).then_some(()).ok_or(Kept::Submodules)?;
        Self::PAUSED_ENTRIES.iter().all(|pause| absent(&[&self.git_dir], pause)).then_some(()).ok_or(Kept::Paused)?;
        let Ok(config) = host.text(&self.git_dir.join(Self::CONFIG)).unwrap_or_default().parse::<WorktreeConfig>();
        config.unredirected()
    }

    fn followed(host: &impl Inspection, file: &Path, base: &Path) -> Option<PathBuf> {
        let Ok(line) = host.text(file)?.parse::<PathLine>();
        host.canonical(&line.resolved(base))
    }
}
