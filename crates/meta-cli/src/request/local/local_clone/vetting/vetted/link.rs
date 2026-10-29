mod path_line;
mod resolved;
mod root_ref;
mod scope;
mod step;
mod worktree_config;

use std::ffi::OsStr;
use std::path::{Path, PathBuf};

use super::ref_line::RefLine;
use super::ref_list::RefList;
use crate::request::local::local_clone::vetting::cause::Cause;
use crate::request::local::local_clone::vetting::context::Context;
use crate::request::local::local_clone::vetting::entry_kind::EntryKind;
use crate::request::local::local_clone::vetting::git_call::GitCall;
use crate::request::local::local_clone::vetting::inspection::Inspection;
use crate::request::local::local_clone::vetting::kept::Kept;
use crate::request::local::local_clone::vetting::thaw::Thaw;
use crate::request::local::worktree_id::WorktreeId;
use path_line::PathLine;
use resolved::Resolved;
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
    const PAUSED_DIRS: [&'static str; 3] = ["rebase-merge", "rebase-apply", "sequencer"];
    const MERGING: &'static str = "MERGE_HEAD";
    const PAUSED_ROOTS: [&'static str; 6] = [
        "for-each-ref",
        "--include-root-refs",
        "--format=%(refname)",
        "CHERRY_PICK_HEAD",
        "REVERT_HEAD",
        "REBASE_HEAD",
    ];
    const CONFIG: &'static str = "config.worktree";
    const REFTABLE: &'static str = "reftable";
    const RESOLVE: [&'static str; 2] = ["cat-file", "--batch-check=%(objectname)"];
    const VERIFY: [&'static str; 3] = ["rev-parse", "--verify", "-q"];
    const HEAD: &'static str = "HEAD";
    const LINE_END: &'static str = "\n";
    const NO_HEAD: &'static str = "HEAD does not resolve";
    const UNEXPECTED: &'static str = "unexpected git cat-file output";
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
        let listed = host.walked(&self.git_dir);
        let named = listed.iter().filter(|found| found.kind() == EntryKind::File);
        let peeled: Vec<String> = named
            .filter_map(|found| found.path().to_str()?.parse::<RootRef>().ok())
            .map(|root| root.peeled())
            .collect();
        let wanted = [Self::HEAD.to_owned()].into_iter().chain(peeled.iter().cloned()).collect::<Vec<_>>();
        let resolved = self.resolved(host, &wanted)?;
        let (head, objects) = resolved.split_first().ok_or_else(|| Kept::Uninspected(Cause::new(Self::NO_HEAD)))?;
        let head = head.as_deref().ok_or_else(|| Kept::Uninspected(Cause::new(Self::NO_HEAD)))?;
        let named = Self::REFS.map(OsStr::new).into_iter().chain([OsStr::new(branch)]).collect::<Vec<_>>();
        let local = self.git(host, Scope::Worktree, &named)?.parse::<RefList>()?.headed(RefLine::head(head));
        let roots =
            peeled.iter().zip(objects).filter_map(|(peeled, object)| Some(RefLine::pseudo(peeled, object.as_deref()?)));
        Ok(roots.fold(local, RefList::headed))
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn registered(&self, host: &impl Inspection) -> bool {
        host.entry(&self.git_dir.join(Self::BACK)) != EntryKind::Missing
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

    fn resolved(&self, host: &impl Inspection, names: &[String]) -> Result<Vec<Option<String>>, Kept> {
        let input = names.iter().flat_map(|name| [name.as_str(), Self::LINE_END]).collect::<String>();
        let call = GitCall::worktree(&self.git_dir, &self.path, &Self::RESOLVE.map(OsStr::new)).fed(input);
        let Ok(resolved) = host.git(&call)?.parse::<Resolved>();
        (resolved.objects().len() == names.len())
            .then(|| resolved.objects().to_vec())
            .ok_or_else(|| Kept::Uninspected(Cause::new(Self::UNEXPECTED)))
    }

    fn names(&self, host: &impl Inspection, name: &str) -> bool {
        let words = Self::VERIFY.map(OsStr::new).into_iter().chain([OsStr::new(name)]).collect::<Vec<_>>();
        self.git(host, Scope::Worktree, &words).is_ok_and(|object| object.lines().next().is_some())
    }

    fn guarded(&self, host: &impl Inspection) -> Result<(), Kept> {
        let absent =
            |dirs: &[&Path], name: &str| dirs.iter().all(|dir| host.entry(&dir.join(name)) == EntryKind::Missing);
        let both = [self.git_dir.as_path(), self.common.as_path()];
        absent(&[&self.git_dir], Self::LOCKED).then_some(()).ok_or(Kept::Locked)?;
        absent(&[&self.git_dir], Self::REFTABLE).then_some(()).ok_or(Kept::Unlistable)?;
        absent(&both, Self::ALTERNATES).then_some(()).ok_or(Kept::Borrowed)?;
        absent(&both, Self::MODULES).then_some(()).ok_or(Kept::Submodules)?;
        Self::PAUSED_DIRS.iter().all(|pause| absent(&[&self.git_dir], pause)).then_some(()).ok_or(Kept::Paused)?;
        (!self.names(host, Self::MERGING)).then_some(()).ok_or(Kept::Paused)?;
        let roots = self.git(host, Scope::Worktree, &Self::PAUSED_ROOTS.map(OsStr::new))?;
        roots.lines().next().is_none().then_some(()).ok_or(Kept::Paused)?;
        let Ok(config) = host.text(&self.git_dir.join(Self::CONFIG)).unwrap_or_default().parse::<WorktreeConfig>();
        config.unredirected()
    }

    fn followed(host: &impl Inspection, file: &Path, base: &Path) -> Option<PathBuf> {
        let Ok(line) = host.text(file)?.parse::<PathLine>();
        host.canonical(&line.resolved(base))
    }
}
