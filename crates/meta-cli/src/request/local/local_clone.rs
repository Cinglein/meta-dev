use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::process::Command;

use meta_log::LogStore;
use meta_schema::{Actor, Claim, Event};
use meta_state::State;

use super::local_settings::LocalSettings;
use super::local_trail::LocalTrail;
use super::recorded_worktree::RecordedWorktree;
use super::trail_entry::TrailEntry;
use super::worktrees::Worktrees;
use crate::request::cli_failure::{CliFailure, Setup};
use crate::request::output::{Listed, Output};
use crate::request::project::Project;
use crate::request::writing::Writing;

#[derive(Debug)]
pub struct LocalClone {
    checkout: PathBuf,
    git_dir: PathBuf,
}

impl LocalClone {
    const LISTED_PREFIX: &'static str = "worktree ";

    pub fn claimed(&self, project: &Project, claim: Claim, actor: Actor) -> Result<Output, CliFailure> {
        let settings = LocalSettings::read(&self.checkout)?;
        let identity = settings.identity()?;
        let task = claim.task();
        let written = Writing::new(&project.log()?, actor, Event::TaskClaimed(claim)).entered()?;
        let branch = task.to_string();
        let path = settings.worktree_root(&self.checkout).join(&branch);
        let base = format!("{}/HEAD", project.config().remote()?);
        let added = ["worktree", "add", "--quiet", "--no-track", "-b", &branch].map(OsStr::new);
        Self::git(&self.checkout, &[&added[..], &[path.as_os_str(), OsStr::new(&base)]].concat())?;
        Self::git(&self.checkout, &["config", "extensions.worktreeConfig", "true"].map(OsStr::new))?;
        let name = identity.name().to_string();
        let email = identity.email().to_string();
        [["user.name", &name], ["user.email", &email]].iter().try_for_each(|[key, value]| {
            Self::git(&path, &["config", "--worktree", key, value].map(OsStr::new)).map(drop)
        })?;
        let created = std::fs::canonicalize(&path).map_err(CliFailure::raised::<Setup>)?;
        LocalTrail::append(&self.git_dir, &TrailEntry::Created(RecordedWorktree::new(created.clone(), task)))?;
        Ok(Output::Listed(Listed::new(format!("{written}{}\n", created.display()))))
    }

    pub fn worktrees(&self, project: &Project) -> Result<Output, CliFailure> {
        let trail = LocalTrail::read(&self.git_dir)?;
        let state = State::fold(project.log()?.read()?.records());
        let listed = Self::git(&self.checkout, &["worktree", "list", "--porcelain"].map(OsStr::new))?;
        let paths: Vec<PathBuf> = listed
            .lines()
            .filter_map(|line| line.strip_prefix(Self::LISTED_PREFIX))
            .map(|path| std::fs::canonicalize(path).unwrap_or_else(|_| PathBuf::from(path)))
            .filter(|path| *path != self.checkout)
            .collect();
        Ok(Output::Listed(Listed::new(Worktrees::joined(&trail, &paths, &state))))
    }

    fn git(dir: &Path, words: &[&OsStr]) -> Result<String, CliFailure> {
        let output =
            Command::new("git").arg("-C").arg(dir).args(words).output().map_err(CliFailure::raised::<Setup>)?;
        let shown = |stream: &[u8]| String::from_utf8_lossy(stream).trim().to_owned();
        output.status.success().then(|| shown(&output.stdout)).ok_or_else(|| {
            let line: Vec<_> = words.iter().map(|word| word.to_string_lossy()).collect();
            CliFailure::raised::<Setup>(format!("git {}: {}", line.join(" "), shown(&output.stderr)))
        })
    }
}

impl TryFrom<&Project> for LocalClone {
    type Error = CliFailure;

    fn try_from(project: &Project) -> Result<Self, Self::Error> {
        let common = ["rev-parse", "--path-format=absolute", "--git-common-dir"].map(OsStr::new);
        let shown = Self::git(project.root(), &common)?;
        let git_dir = std::fs::canonicalize(shown).map_err(CliFailure::raised::<Setup>)?;
        let checkout =
            git_dir.parent().map(Path::to_path_buf).ok_or_else(|| CliFailure::raised::<Setup>("no checkout"))?;
        Ok(Self { checkout, git_dir })
    }
}
