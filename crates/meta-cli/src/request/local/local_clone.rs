mod git;

use std::ffi::OsStr;
use std::path::{Path, PathBuf};

use chrono::Utc;
use meta_log::{LogStore, RemoteName};
use meta_schema::{Actor, Claim, Event, Record};
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
use git::Git;

#[derive(Debug)]
pub struct LocalClone {
    checkout: PathBuf,
    git_dir: PathBuf,
}

impl LocalClone {
    const LISTED_PREFIX: &'static str = "worktree ";

    pub fn claimed(&self, project: &Project, claim: Claim, actor: Actor) -> Result<Output, CliFailure> {
        let settings = self.settings()?;
        let identity = settings.identity()?;
        let log = project.log()?;
        let state = State::fold(log.read()?.records());
        let task = claim.task();
        let held = state.holders(task).contains(&actor).then_some(());
        let event = Event::TaskClaimed(claim);
        held.map_or_else(|| state.vet(&Record::new(actor.clone(), event.clone())), Ok)?;
        let branch = task.to_string();
        let path = settings.worktree_root(&self.checkout).join(&branch);
        let base = self.base(project.config().remote()?, &branch, held)?;
        Some(())
            .filter(|()| !path.exists())
            .ok_or_else(|| CliFailure::raised::<Setup>(format!("{} exists; move it away first", path.display())))?;
        let checkout = Git::new(&self.checkout);
        checkout.output(&["worktree", "prune"].map(OsStr::new))?;
        checkout.output(&["config", "extensions.worktreeConfig", "true"].map(OsStr::new))?;
        let new_branch = base.as_ref().map(|_| branch.as_str());
        let placed = base.as_ref().map_or_else(
            || vec![path.as_os_str(), OsStr::new(&branch)],
            |base| vec![OsStr::new("-b"), OsStr::new(&branch), path.as_os_str(), OsStr::new(base)],
        );
        let name = identity.name().to_string();
        let email = identity.email().to_string();
        let added = || {
            let worktree = Git::new(&path);
            checkout
                .output(&[&["worktree", "add", "--quiet", "--no-checkout"].map(OsStr::new)[..], &placed].concat())?;
            worktree.output(&["config", "--worktree", "user.name", &name].map(OsStr::new))?;
            worktree.output(&["config", "--worktree", "user.email", &email].map(OsStr::new))?;
            worktree.output(&["reset", "--hard", "--quiet"].map(OsStr::new))?;
            std::fs::canonicalize(&path).map_err(CliFailure::raised::<Setup>)
        };
        let created = added().map_err(|failure| self.rolled_back(&path, new_branch, failure))?;
        LocalTrail::append(&self.git_dir, &TrailEntry::Created(RecordedWorktree::new(created.clone(), task)))?;
        let written = held.map_or_else(
            || {
                Writing::new(&log, actor, event).entered().map(|written| written.to_string()).map_err(|failure| {
                    let removed = TrailEntry::Removed { path: created.clone(), at: Utc::now() };
                    self.rolled_back(
                        &path,
                        new_branch,
                        LocalTrail::append(&self.git_dir, &removed).err().unwrap_or(failure),
                    )
                })
            },
            |()| Ok(String::new()),
        )?;
        Ok(Output::Listed(Listed::new(format!("{written}{}\n", created.display()))))
    }

    pub fn worktrees(&self, project: &Project) -> Result<Output, CliFailure> {
        let trail = LocalTrail::read(&self.git_dir)?;
        let state = State::fold(project.log()?.read()?.records());
        let listed = Git::new(&self.checkout).output(&["worktree", "list", "--porcelain"].map(OsStr::new))?;
        let paths: Vec<PathBuf> = listed
            .lines()
            .filter_map(|line| line.strip_prefix(Self::LISTED_PREFIX))
            .map(|path| std::fs::canonicalize(path).unwrap_or_else(|_| PathBuf::from(path)))
            .filter(|path| *path != self.checkout)
            .collect();
        Ok(Output::Listed(Listed::new(Worktrees::joined(trail, &paths, &state))))
    }

    fn settings(&self) -> Result<LocalSettings, CliFailure> {
        let ignored = ["check-ignore", "--quiet", LocalSettings::FILE].map(OsStr::new);
        Some(())
            .filter(|()| self.checkout.join(LocalSettings::FILE).exists())
            .map_or(Some(()), |()| Git::new(&self.checkout).holds(&ignored))
            .ok_or_else(|| {
                CliFailure::raised::<Setup>(format!(
                    "{file} is not gitignored; add /{file} to .gitignore",
                    file = LocalSettings::FILE
                ))
            })?;
        LocalSettings::read(&self.checkout)
    }

    fn base(&self, remote: &RemoteName, branch: &str, held: Option<()>) -> Result<Option<String>, CliFailure> {
        let checkout = Git::new(&self.checkout);
        checkout.output(&["fetch", "--quiet", &remote.to_string()].map(OsStr::new))?;
        let verified = |name: &str| checkout.holds(&["rev-parse", "--verify", "--quiet", name].map(OsStr::new));
        match verified(&format!("refs/heads/{branch}")) {
            Some(()) => held.map(|()| None).ok_or_else(|| {
                CliFailure::raised::<Setup>(format!("branch {branch} exists but you do not hold the task"))
            }),
            None => [format!("{remote}/{branch}"), format!("{remote}/HEAD")]
                .into_iter()
                .find(|base| verified(base).is_some())
                .map(Some)
                .ok_or_else(|| {
                    CliFailure::raised::<Setup>(format!(
                        "{remote}/HEAD is unset; run git remote set-head {remote} --auto"
                    ))
                }),
        }
    }

    fn rolled_back(&self, path: &Path, new_branch: Option<&str>, failure: CliFailure) -> CliFailure {
        let checkout = Git::new(&self.checkout);
        checkout.holds(&[OsStr::new("worktree"), OsStr::new("remove"), OsStr::new("--force"), path.as_os_str()]);
        new_branch.and_then(|branch| checkout.holds(&["branch", "-D", branch].map(OsStr::new)));
        failure
    }
}

impl TryFrom<&Project> for LocalClone {
    type Error = CliFailure;

    fn try_from(project: &Project) -> Result<Self, Self::Error> {
        let common = ["rev-parse", "--path-format=absolute", "--git-common-dir"].map(OsStr::new);
        let shown = Git::new(project.root()).output(&common)?;
        let git_dir = std::fs::canonicalize(shown).map_err(CliFailure::raised::<Setup>)?;
        let checkout =
            git_dir.parent().map(Path::to_path_buf).ok_or_else(|| CliFailure::raised::<Setup>("no checkout"))?;
        Ok(Self { checkout, git_dir })
    }
}
