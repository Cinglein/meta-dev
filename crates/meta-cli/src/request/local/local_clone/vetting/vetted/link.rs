mod path_line;
mod scope;
mod worktree_config;

use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};

use super::ref_line::RefLine;
use crate::request::local::local_clone::vetting::context::Context;
use crate::request::local::local_clone::vetting::entry_kind::EntryKind;
use crate::request::local::local_clone::vetting::git_call::GitCall;
use crate::request::local::local_clone::vetting::inspection::Inspection;
use crate::request::local::local_clone::vetting::kept::Kept;
use crate::request::local::worktree_id::WorktreeId;
use path_line::PathLine;
pub use scope::Scope;
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
    const PAUSES: [&'static str; 6] =
        ["rebase-merge", "rebase-apply", "sequencer", "MERGE_HEAD", "CHERRY_PICK_HEAD", "REVERT_HEAD"];
    const CONFIG: &'static str = "config.worktree";
    const ORIG_HEAD: &'static str = "ORIG_HEAD";
    const QUARANTINE: &'static str = ".gc-";
    const REMOVE: [&'static str; 2] = ["worktree", "remove"];
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

    pub fn orig_head(&self, host: &impl Inspection) -> Option<RefLine> {
        host.text(&self.git_dir.join(Self::ORIG_HEAD)).map(|object| RefLine::pseudo(Self::ORIG_HEAD, &object))
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn quarantine(&self) -> PathBuf {
        let mut name = OsString::from(Self::QUARANTINE);
        name.push(self.path.file_name().unwrap_or_default());
        self.path.with_file_name(name)
    }

    pub fn relocated(&self, path: &Path) -> Self {
        Self { path: path.to_path_buf(), ..self.clone() }
    }

    pub fn moving(&self, to: &Path) -> GitCall {
        let words = Self::MOVE.map(OsStr::new).into_iter().chain([self.path.as_os_str(), to.as_os_str()]);
        GitCall::common(&self.common, &words.collect::<Vec<_>>())
    }

    pub fn removal(&self) -> GitCall {
        let words = Self::REMOVE.map(OsStr::new).into_iter().chain([self.path.as_os_str()]).collect::<Vec<_>>();
        GitCall::common(&self.common, &words)
    }

    pub fn unbranch(&self, branch: &RefLine) -> GitCall {
        let words = Self::UNBRANCH.map(OsStr::new).into_iter().chain(branch.deletion()).collect::<Vec<_>>();
        GitCall::common(&self.common, &words)
    }

    fn guarded(&self, host: &impl Inspection) -> Result<(), Kept> {
        let absent =
            |dirs: &[&Path], name: &str| dirs.iter().all(|dir| host.entry(&dir.join(name)) == EntryKind::Missing);
        let both = [self.git_dir.as_path(), self.common.as_path()];
        absent(&[&self.git_dir], Self::LOCKED).then_some(()).ok_or(Kept::Locked)?;
        absent(&both, Self::ALTERNATES).then_some(()).ok_or(Kept::Borrowed)?;
        absent(&both, Self::MODULES).then_some(()).ok_or(Kept::Submodules)?;
        Self::PAUSES.iter().all(|pause| absent(&[&self.git_dir], pause)).then_some(()).ok_or(Kept::Paused)?;
        let Ok(config) = host.text(&self.git_dir.join(Self::CONFIG)).unwrap_or_default().parse::<WorktreeConfig>();
        config.unredirected()
    }

    fn followed(host: &impl Inspection, file: &Path, base: &Path) -> Option<PathBuf> {
        let Ok(line) = host.text(file)?.parse::<PathLine>();
        host.canonical(&line.resolved(base))
    }
}
