mod path_line;

use std::ffi::OsStr;
use std::path::{Path, PathBuf};

use super::ref_line::RefLine;
use crate::request::local::local_clone::vetting::context::Context;
use crate::request::local::local_clone::vetting::entry_kind::EntryKind;
use crate::request::local::local_clone::vetting::git_call::GitCall;
use crate::request::local::local_clone::vetting::inspection::Inspection;
use crate::request::local::local_clone::vetting::kept::Kept;
use path_line::PathLine;

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
    const REMOVE: [&'static str; 2] = ["worktree", "remove"];
    const UNBRANCH: [&'static str; 2] = ["update-ref", "-d"];

    pub fn proved(path: &Path, context: &Context, host: &impl Inspection) -> Result<Self, Kept> {
        let entry = path.join(Self::ENTRY);
        (host.entry(&entry) == EntryKind::File).then_some(()).ok_or(Kept::Unlinked)?;
        let common = context.common();
        let git_dir = Self::followed(host, &entry, path)
            .filter(|git_dir| git_dir.parent() == Some(common.join(Self::LINKED).as_path()))
            .ok_or(Kept::Unlinked)?;
        (Self::followed(host, &git_dir.join(Self::BACK), &git_dir) == Some(entry))
            .then_some(())
            .ok_or(Kept::Escaped)?;
        let shared = Self::followed(host, &git_dir.join(Self::COMMON), &git_dir);
        (shared.as_deref() == Some(common)).then_some(()).ok_or(Kept::Escaped)?;
        (host.entry(&git_dir.join(Self::LOCKED)) == EntryKind::Missing).then_some(()).ok_or(Kept::Locked)?;
        let absent =
            |name: &str| [&git_dir, common].iter().all(|dir| host.entry(&dir.join(name)) == EntryKind::Missing);
        absent(Self::ALTERNATES).then_some(()).ok_or(Kept::Borrowed)?;
        absent(Self::MODULES).then_some(()).ok_or(Kept::Submodules)?;
        Ok(Self { path: path.to_path_buf(), git_dir, common: common.to_path_buf() })
    }

    pub fn git(&self, host: &impl Inspection, words: &[&OsStr]) -> Result<String, Kept> {
        host.git(&GitCall::worktree(&self.git_dir, &self.path, words))
    }

    pub fn removal(&self) -> GitCall {
        let words = Self::REMOVE.map(OsStr::new).into_iter().chain([self.path.as_os_str()]).collect::<Vec<_>>();
        GitCall::common(&self.common, &words)
    }

    pub fn unbranch(&self, branch: &RefLine) -> GitCall {
        let words = Self::UNBRANCH.map(OsStr::new).into_iter().chain(branch.deletion()).collect::<Vec<_>>();
        GitCall::common(&self.common, &words)
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    fn followed(host: &impl Inspection, file: &Path, base: &Path) -> Option<PathBuf> {
        let Ok(line) = host.text(file)?.parse::<PathLine>();
        host.canonical(&line.resolved(base))
    }
}
