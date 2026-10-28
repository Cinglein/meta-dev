mod collection;
mod gc_lock;
mod git;
mod quarantine;
mod vetting;
mod worktree_identity;

use std::ffi::OsStr;
use std::path::{Path, PathBuf};

use chrono::Utc;
use meta_door::kinds::Kind;
use meta_door::{Door, MetaStore};
use meta_log::{LogStore, RemoteName};
use meta_schema::{Actor, Claim, Event, Id, Record, Task};
use meta_state::State;

use meta_status::Status;

use super::local_settings::{LocalSettings, MonitorSettings};
use super::local_trail::LocalTrail;
use super::recorded_worktree::RecordedWorktree;
use super::sweep::Sweep;
use super::trail_entry::TrailEntry;
use super::worktree_id::WorktreeId;
use super::worktrees::Worktrees;
use crate::request::cli_failure::{CliFailure, Setup};
use crate::request::live_github::LiveGithub;
use crate::request::output::{Listed, Output};
use crate::request::project::Project;
use crate::request::writing::Writing;
use collection::{Census, Collection, Sweeper, Tracked};
use gc_lock::GcLock;
use git::Git;
use vetting::{Context, Deletable, Host};
use worktree_identity::WorktreeIdentity;

#[derive(Debug)]
pub struct LocalClone {
    checkout: PathBuf,
    store: MetaStore,
}

impl LocalClone {
    const LISTED_PREFIX: &'static str = "worktree ";

    pub fn new(checkout: PathBuf, store: MetaStore) -> Self {
        Self { checkout, store }
    }

    pub fn checkout(&self) -> &Path {
        &self.checkout
    }

    pub fn store(&self) -> &MetaStore {
        &self.store
    }

    pub fn monitor_settings(&self) -> Result<MonitorSettings, CliFailure> {
        self.settings()?.monitor(&self.checkout)
    }

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
            .filter(|()| Door::described(&path).is_err())
            .ok_or_else(|| CliFailure::raised::<Setup>(format!("{} exists; move it away first", path.display())))?;
        let checkout = Git::new(&self.checkout);
        checkout.output(&["worktree", "prune"].map(OsStr::new))?;
        checkout.output(&["config", "extensions.worktreeConfig", "true"].map(OsStr::new))?;
        let new_branch = base
            .map(|base| {
                checkout.output(&["branch", "--no-track", &branch, &base].map(OsStr::new)).map(|_| branch.as_str())
            })
            .transpose()?;
        let added = ["worktree", "add", "--quiet", "--no-checkout"].map(OsStr::new);
        checkout
            .output(&[&added[..], &[path.as_os_str(), OsStr::new(&branch)]].concat())
            .map_err(|failure| self.rolled_back(None, new_branch, failure))?;
        let name = identity.name().to_string();
        let email = identity.email().to_string();
        let settled = || {
            let worktree = Git::new(&path);
            let git_dir = worktree.output(&["rev-parse", "--absolute-git-dir"].map(OsStr::new))?;
            WorktreeIdentity::new(&name, &email).configured(&worktree)?;
            worktree.output(&["reset", "--hard", "--quiet"].map(OsStr::new))?;
            let created = Door::canonical(&path).map_err(CliFailure::raised::<Setup>)?;
            let id =
                WorktreeId::parsed(Path::new(&git_dir)).ok_or_else(|| CliFailure::raised::<Setup>("no worktree id"))?;
            let recorded = RecordedWorktree::new(created.clone(), task).identified(id);
            LocalTrail::append(&self.store, &TrailEntry::Created(recorded))?;
            Ok(created)
        };
        let created = settled().map_err(|failure| self.rolled_back(Some(&path), new_branch, failure))?;
        let written = held.map_or_else(
            || {
                Writing::new(&log, actor, event).entered().map(|written| written.to_string()).map_err(|failure| {
                    let removed = TrailEntry::Removed { path: created.clone(), at: Utc::now() };
                    let failure = LocalTrail::append(&self.store, &removed).err().unwrap_or(failure);
                    self.rolled_back(Some(&path), new_branch, failure)
                })
            },
            |()| Ok(String::new()),
        )?;
        Ok(Output::Listed(Listed::new(format!("{written}{}\n", created.display()))))
    }

    pub fn worktrees(&self, project: &Project) -> Result<Output, CliFailure> {
        let trail = LocalTrail::read(&self.store)?;
        let state = State::fold(project.log()?.read()?.records());
        let paths = self.listed()?;
        trail.torn().iter().for_each(|torn| eprintln!("meta: warning: {torn}"));
        Ok(Output::Listed(Listed::new(Worktrees::joined(&trail, &paths, &state))))
    }

    pub fn collected(&self, project: &Project, sweep: Sweep) -> Result<Output, CliFailure> {
        let root = self.settings()?.worktree_root(&self.checkout);
        let _held = GcLock::taken(&self.store)?;
        let trail = LocalTrail::read(&self.store)?;
        trail.torn().iter().for_each(|torn| eprintln!("meta: warning: {torn}"));
        let state = State::fold(project.log()?.read()?.records());
        let recorded: Vec<RecordedWorktree> = trail.current();
        let tasks: Vec<Id<Task>> = recorded.iter().map(RecordedWorktree::task).collect();
        let github = LiveGithub::for_project(project).read(&Status::linked_query(&state, &tasks));
        let status = Status::new(&state, &github, Utc::now().into());
        let tracked = recorded.into_iter().map(|worktree| {
            let lifetime = status.lifetime(worktree.task());
            Tracked::new(worktree, lifetime)
        });
        let census = Census::new(tracked.collect(), self.listed()?);
        let root = Door::canonical(&root).unwrap_or(root);
        let context = Context::new(
            self.checkout.clone(),
            self.store.common().to_path_buf(),
            root,
            project.config().remote()?.clone(),
        );
        let collection = Collection::swept(&Sweeper::new(sweep, &context, &Host::default()), &census);
        collection.trailed(Utc::now()).iter().try_for_each(|entry| LocalTrail::append(&self.store, entry))?;
        Ok(Output::Listed(Listed::new(collection)))
    }

    pub fn unstranded(&self, quarantine: &Path) -> Result<Output, CliFailure> {
        let quarantine = Door::canonical(quarantine).map_err(CliFailure::raised::<Setup>)?;
        let origin = Deletable::unstranded(&quarantine, self.store.common(), &Host::default())
            .map_err(CliFailure::raised::<Setup>)?;
        Ok(Output::Listed(Listed::new(format!("restored {} to {}\n", quarantine.display(), origin.display()))))
    }

    fn listed(&self) -> Result<Vec<PathBuf>, CliFailure> {
        let listed = Git::new(&self.checkout).output(&["worktree", "list", "--porcelain"].map(OsStr::new))?;
        Ok(listed
            .lines()
            .filter_map(|line| line.strip_prefix(Self::LISTED_PREFIX))
            .map(|path| Door::canonical(path).unwrap_or_else(|_| PathBuf::from(path)))
            .filter(|path| *path != self.checkout)
            .collect())
    }

    fn settings(&self) -> Result<LocalSettings, CliFailure> {
        let ignored = ["check-ignore", "--quiet", LocalSettings::FILE].map(OsStr::new);
        Some(())
            .filter(|()| Door::kind(self.checkout.join(LocalSettings::FILE)) != Kind::Missing)
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

    fn rolled_back(&self, added: Option<&Path>, new_branch: Option<&str>, failure: CliFailure) -> CliFailure {
        let checkout = Git::new(&self.checkout);
        added.and_then(|path| {
            checkout.holds(&[OsStr::new("worktree"), OsStr::new("remove"), OsStr::new("--force"), path.as_os_str()])
        });
        new_branch.and_then(|branch| checkout.holds(&["branch", "-D", branch].map(OsStr::new)));
        failure
    }
}

impl TryFrom<&Project> for LocalClone {
    type Error = CliFailure;

    fn try_from(project: &Project) -> Result<Self, Self::Error> {
        let store = MetaStore::located(project.root()).map_err(CliFailure::raised::<Setup>)?;
        let checkout =
            store.common().parent().map(Path::to_path_buf).ok_or_else(|| CliFailure::raised::<Setup>("no checkout"))?;
        Ok(Self::new(checkout, store))
    }
}
