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
use crate::request::local::local_clone::vetting::halt::Halt;
use crate::request::local::local_clone::vetting::kept::Kept;
use crate::request::local::local_clone::vetting::look::Look;
use crate::request::local::local_clone::vetting::replay::Replay;
use crate::request::local::local_clone::vetting::seen::Seen;
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
    const PAUSED: [&'static str; 7] = [
        "rebase-merge",
        "rebase-apply",
        "sequencer",
        "MERGE_HEAD",
        "CHERRY_PICK_HEAD",
        "REVERT_HEAD",
        "REBASE_HEAD",
    ];
    const CONFIG: &'static str = "config.worktree";
    const REFTABLE: &'static str = "reftable";
    const RESOLVE: [&'static str; 2] = ["cat-file", "--batch-check=%(objectname)"];
    const HEAD: &'static str = "HEAD";
    const LINE_END: &'static str = "\n";
    const NO_HEAD: &'static str = "HEAD does not resolve";
    const UNEXPECTED: &'static str = "unexpected git cat-file output";
    const REFS: [&'static str; 5] =
        ["for-each-ref", "--format=%(objectname) %(refname)", "refs/worktree/", "refs/bisect/", "refs/rewritten/"];
    const MOVE: [&'static str; 2] = ["worktree", "move"];
    const UNBRANCH: [&'static str; 2] = ["update-ref", "-d"];

    // The chain is data-dependent: each round can name only what the last one revealed.
    pub fn proved(
        path: &Path,
        id: Option<&WorktreeId>,
        context: &Context,
        replay: &Replay<'_>,
    ) -> Result<Self, Halt> {
        let entry = path.join(Self::ENTRY);
        let [kind, text] = Self::batch(replay, [Look::Entry(entry.clone()), Look::Text(entry.clone())])?;
        (kind.entry()? == EntryKind::File).then_some(()).ok_or(Kept::Unlinked)?;
        let common = context.common();
        let pointed = text.text()?.map(|text| Self::pointed(&text, path));
        let git_dir = match pointed {
            Some(pointed) => replay.canonical(&pointed)?,
            None => None,
        };
        let git_dir =
            git_dir.filter(|git_dir| git_dir.parent() == Some(common.join(Self::LINKED).as_path())).ok_or(Kept::Unlinked)?;
        id.ok_or(Kept::Unidentified)?.names(&git_dir).then_some(()).ok_or(Kept::Stranger)?;
        let [back, shared] = Self::followed(replay, [git_dir.join(Self::BACK), git_dir.join(Self::COMMON)], &git_dir)?;
        (back == Some(entry)).then_some(()).ok_or(Kept::Escaped)?;
        (shared.as_deref() == Some(common)).then_some(()).ok_or(Kept::Escaped)?;
        let link = Self { path: path.to_path_buf(), git_dir, common: common.to_path_buf() };
        link.guarded(replay).map(|()| link)
    }

    pub fn call(&self, scope: Scope, words: &[&OsStr]) -> GitCall {
        match scope {
            Scope::Worktree => GitCall::worktree(&self.git_dir, &self.path, words),
            Scope::Common => GitCall::common(&self.common, words),
        }
    }

    pub fn git(&self, replay: &Replay<'_>, scope: Scope, words: &[&OsStr]) -> Result<Result<String, Kept>, Halt> {
        replay.git(self.call(scope, words))
    }

    pub fn refs(&self, replay: &Replay<'_>, branch: &str) -> Result<RefList, Halt> {
        let named = Self::REFS.map(OsStr::new).into_iter().chain([OsStr::new(branch)]).collect::<Vec<_>>();
        let local = Look::Git(self.call(Scope::Worktree, &named));
        let [walked, local] = Self::batch(replay, [Look::Walk(self.git_dir.clone()), local])?;
        let listed = walked.walked()?;
        let named = listed.iter().filter(|found| found.kind() == EntryKind::File);
        let peeled: Vec<String> = named
            .filter_map(|found| found.path().to_str()?.parse::<RootRef>().ok())
            .map(|root| root.peeled())
            .collect();
        let wanted = [Self::HEAD.to_owned()].into_iter().chain(peeled.iter().cloned()).collect::<Vec<_>>();
        let resolved = self.resolved(replay, &wanted)?;
        let (head, objects) = resolved.split_first().ok_or_else(|| Kept::Uninspected(Cause::new(Self::NO_HEAD)))?;
        let head = head.as_deref().ok_or_else(|| Kept::Uninspected(Cause::new(Self::NO_HEAD)))?;
        let local = local.git()??.parse::<RefList>()?.headed(RefLine::head(head));
        let roots =
            peeled.iter().zip(objects).filter_map(|(peeled, object)| Some(RefLine::pseudo(peeled, object.as_deref()?)));
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

    fn batch<const N: usize>(replay: &Replay<'_>, looks: [Look; N]) -> Result<[Seen; N], Halt> {
        <[Seen; N]>::try_from(replay.looked(looks.to_vec())?).map_err(|_| Replay::diverged())
    }

    fn pointed(text: &str, base: &Path) -> PathBuf {
        let Ok(line) = text.parse::<PathLine>();
        line.resolved(base)
    }

    fn resolved(&self, replay: &Replay<'_>, names: &[String]) -> Result<Vec<Option<String>>, Halt> {
        let input = names.iter().flat_map(|name| [name.as_str(), Self::LINE_END]).collect::<String>();
        let call = GitCall::worktree(&self.git_dir, &self.path, &Self::RESOLVE.map(OsStr::new)).fed(input);
        let Ok(resolved) = replay.git(call)??.parse::<Resolved>();
        Ok((resolved.objects().len() == names.len())
            .then(|| resolved.objects().to_vec())
            .ok_or_else(|| Kept::Uninspected(Cause::new(Self::UNEXPECTED)))?)
    }

    fn guarded(&self, replay: &Replay<'_>) -> Result<(), Halt> {
        let both = [self.git_dir.as_path(), self.common.as_path()];
        let own = [self.git_dir.as_path()];
        // A pause is a file in the worktree's git dir (reftable is refused above), so a present
        // MERGE_HEAD or *_HEAD keeps the worktree whatever it holds: no git process needed.
        let groups: [(&[&Path], &[&str]); 5] = [
            (&own, &[Self::LOCKED]),
            (&own, &[Self::REFTABLE]),
            (&both, &[Self::ALTERNATES]),
            (&both, &[Self::MODULES]),
            (&own, &Self::PAUSED),
        ];
        let entries = groups.iter().flat_map(|(dirs, names)| {
            names.iter().flat_map(move |name| dirs.iter().map(move |dir| Look::Entry(dir.join(name))))
        });
        let looks: Vec<Look> = entries.chain([Look::Text(self.git_dir.join(Self::CONFIG))]).collect();
        let mut seen = replay.looked(looks)?.into_iter();
        let mut absent = |count: usize| -> Result<bool, Halt> {
            seen.by_ref().take(count).try_fold(true, |all, seen| Ok(all && seen.entry()? == EntryKind::Missing))
        };
        let verdicts = [Kept::Locked, Kept::Unlistable, Kept::Borrowed, Kept::Submodules, Kept::Paused];
        let mut found = None;
        for ((dirs, names), kept) in groups.iter().zip(verdicts) {
            let clear = absent(dirs.len() * names.len())?;
            found = found.or((!clear).then_some(kept));
        }
        found.map_or(Ok(()), Err)?;
        let config = seen.next().ok_or_else(Replay::diverged)?.text()?;
        let Ok(config) = config.unwrap_or_default().parse::<WorktreeConfig>();
        Ok(config.unredirected()?)
    }

    fn followed<const N: usize>(
        replay: &Replay<'_>,
        files: [PathBuf; N],
        base: &Path,
    ) -> Result<[Option<PathBuf>; N], Halt> {
        let texts = Self::batch(replay, files.map(Look::Text))?;
        let pointed: Vec<Option<PathBuf>> =
            texts.into_iter().map(|text| Ok(text.text()?.map(|text| Self::pointed(&text, base)))).collect::<Result<_, Halt>>()?;
        let looks: Vec<Look> = pointed.iter().flatten().cloned().map(Look::Canonical).collect();
        let mut canonical = if looks.is_empty() { Vec::new() } else { replay.looked(looks)? }.into_iter();
        let followed: Vec<Option<PathBuf>> = pointed
            .iter()
            .map(|pointed| match pointed {
                Some(_) => canonical.next().ok_or_else(Replay::diverged)?.canonical(),
                None => Ok(None),
            })
            .collect::<Result<_, Halt>>()?;
        <[Option<PathBuf>; N]>::try_from(followed).map_err(|_| Replay::diverged())
    }
}
