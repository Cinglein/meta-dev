mod claim_plan;
mod claim_run;
mod collection;
mod gc_lock;
mod git;
mod git_dir;
mod quarantine;
mod vetting;
mod worktree_listing;

use std::ffi::OsStr;
use std::path::{Path, PathBuf};

use chrono::Utc;
use meta_log::LogStore;
use meta_run::GitArgv;
use meta_schema::{Actor, Claim, Event, Id, Record, Task};
use meta_state::State;

use meta_status::Status;

use super::local_settings::{LocalSettings, MonitorSettings};
use super::local_trail::LocalTrail;
use super::recorded_worktree::RecordedWorktree;
use super::sweep::Sweep;
use super::worktrees::Worktrees;
use crate::request::cli_failure::{CliFailure, Setup};
use crate::request::git_link::GitLink;
use crate::request::live_github::LiveGithub;
use crate::request::output::{Listed, Output};
use crate::request::project::{BranchName, CiSettings, Project};
use crate::request::writing::Writing;
use claim_plan::{BranchRefs, ClaimPlan, Holding};
use claim_run::ClaimRun;
use collection::{Census, Collection, Sweeper, Tracked};
use gc_lock::GcLock;
use git::Git;
use git_dir::GitDir;
use vetting::{Context, Deletable, Host};
use worktree_listing::WorktreeListing;

#[derive(Debug)]
pub struct LocalClone {
    checkout: PathBuf,
    git_dir: GitDir,
    settings: CiSettings,
}

impl LocalClone {
    pub fn new(checkout: PathBuf, git_dir: PathBuf, settings: CiSettings) -> Self {
        Self { checkout, git_dir: GitDir::new(git_dir), settings }
    }

    pub fn checkout(&self) -> &Path {
        &self.checkout
    }

    pub fn git_dir(&self) -> &GitDir {
        &self.git_dir
    }

    pub fn monitor_settings(&self) -> Result<MonitorSettings, CliFailure> {
        self.settings()?.monitor(&self.checkout)
    }

    pub fn claimed(&self, project: &Project, claim: Claim, actor: Actor) -> Result<Output, CliFailure> {
        let settings = self.settings()?;
        let identity = settings.identity()?;
        let log = project.log(project.ci_settings().budget(CiSettings::ROUTINE))?;
        let task = claim.task();
        let record = Record::new(actor, Event::TaskClaimed(claim));
        let holding = Holding::vetted(&State::fold(log.read()?.records()), &record, task)?;
        let refs = self.refs(project, task)?;
        let worktree =
            RecordedWorktree::new(settings.worktree_root(&self.checkout).join(OsStr::new(refs.branch())), task);
        let taken = Some(worktree.path()).filter(|path| path.exists());
        let plan = ClaimPlan::new(holding, &refs, taken).map_err(CliFailure::raised::<Setup>)?;
        let written = || {
            let writing = Writing::new(&log, record.actor().clone(), record.event().clone());
            writing.entered().map(|written| written.to_string())
        };
        let run = ClaimRun::new(self, worktree.clone(), identity, written);
        let shown = run.ran(&plan)?;
        Ok(Output::Listed(Listed::new(format!("{shown}{}\n", run.created()?.display()))))
    }

    pub fn worktrees(&self, project: &Project) -> Result<Output, CliFailure> {
        let trail = LocalTrail::read(self.git_dir.as_ref())?;
        let state = State::fold(project.log(project.ci_settings().budget(CiSettings::ROUTINE))?.read()?.records());
        let paths = self.listed()?;
        trail.torn().iter().for_each(|torn| eprintln!("meta: warning: {torn}"));
        Ok(Output::Listed(Listed::new(Worktrees::joined(&trail, &paths, &state))))
    }

    pub fn collected(&self, project: &Project, sweep: Sweep) -> Result<Output, CliFailure> {
        let root = self.settings()?.worktree_root(&self.checkout);
        let _held = GcLock::taken(self.git_dir.as_ref())?;
        let trail = LocalTrail::read(self.git_dir.as_ref())?;
        trail.torn().iter().for_each(|torn| eprintln!("meta: warning: {torn}"));
        let state = State::fold(project.log(project.ci_settings().budget(CiSettings::ROUTINE))?.read()?.records());
        let recorded: Vec<RecordedWorktree> = trail.current();
        let tasks: Vec<Id<Task>> = recorded.iter().map(RecordedWorktree::task).collect();
        let github = LiveGithub::for_project(project).read(&Status::linked_query(&state, &tasks));
        let status = Status::new(&state, &github, Utc::now().into());
        let tracked = recorded.into_iter().map(|worktree| {
            let lifetime = status.lifetime(worktree.task());
            Tracked::new(worktree, lifetime)
        });
        let root = std::fs::canonicalize(&root).unwrap_or(root);
        let context = Context::new(
            self.checkout.clone(),
            self.git_dir.as_ref().to_path_buf(),
            root,
            project.config().remote()?.clone(),
        );
        let host = Host::new(self.settings.clone());
        let listed = context.worktrees(&host).as_ref().map_err(CliFailure::raised::<Setup>)?;
        let census = Census::new(tracked.collect(), listed.paths(&self.checkout));
        let collection = Collection::swept(&Sweeper::new(sweep, &context, &host), &census);
        collection.trailed(Utc::now()).iter().try_for_each(|entry| LocalTrail::append(self.git_dir.as_ref(), entry))?;
        Ok(Output::Listed(Listed::new(collection)))
    }

    pub fn unstranded(&self, quarantine: &Path) -> Result<Output, CliFailure> {
        let quarantine = std::fs::canonicalize(quarantine).map_err(CliFailure::raised::<Setup>)?;
        let origin = Deletable::unstranded(&quarantine, self.git_dir.as_ref(), &Host::new(self.settings.clone()))
            .map_err(CliFailure::raised::<Setup>)?;
        Ok(Output::Listed(Listed::new(format!("restored {} to {}\n", quarantine.display(), origin.display()))))
    }

    fn listed(&self) -> Result<Vec<PathBuf>, CliFailure> {
        let listed = Git::from(self).output(GitArgv::WorktreeList, &[])?;
        Ok(WorktreeListing::from(listed).paths(&self.checkout))
    }

    fn settings(&self) -> Result<LocalSettings, CliFailure> {
        let present = Some(()).filter(|()| self.checkout.join(LocalSettings::FILE).exists());
        let ignored = || Git::from(self).holds(GitArgv::CheckIgnore, &[OsStr::new(LocalSettings::FILE)]);
        LocalSettings::admitted(present, ignored)?;
        LocalSettings::read(&self.checkout)
    }

    fn refs(&self, project: &Project, task: Id<Task>) -> Result<BranchRefs, CliFailure> {
        let remote = project.config().remote()?.clone();
        let checkout = Git::from(self);
        checkout.output(GitArgv::Fetch, &[remote.as_ref()])?;
        let verified = |name: &OsStr| checkout.holds(GitArgv::VerifyRef, &[name]);
        Ok(BranchRefs::gathered(remote, BranchName::from(&task), verified))
    }
}

impl TryFrom<&Project> for LocalClone {
    type Error = CliFailure;

    fn try_from(project: &Project) -> Result<Self, Self::Error> {
        let git_dir =
            GitLink::common_dir(project.root()).ok_or_else(|| CliFailure::raised::<Setup>("no git directory"))?;
        let checkout =
            git_dir.parent().map(Path::to_path_buf).ok_or_else(|| CliFailure::raised::<Setup>("no checkout"))?;
        Ok(Self::new(checkout, git_dir, project.ci_settings().clone()))
    }
}
