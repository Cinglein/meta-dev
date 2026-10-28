mod worktree_identity;

use std::path::PathBuf;

use chrono::Utc;
use meta_run::GitArgv;

use super::LocalClone;
use super::claim_plan::{ClaimHands, ClaimPlan, ClaimStep};
use super::git::Git;
use crate::request::cli_failure::{CliFailure, Setup};
use crate::request::local::local_settings::Identity;
use crate::request::local::local_trail::LocalTrail;
use crate::request::local::recorded_worktree::RecordedWorktree;
use crate::request::local::trail_entry::TrailEntry;
use crate::request::local::worktree_id::WorktreeId;
use crate::request::project::BranchName;
use worktree_identity::WorktreeIdentity;

pub struct ClaimRun<'claim, Write: Fn() -> Result<String, CliFailure>> {
    clone: &'claim LocalClone,
    checkout: Git<'claim>,
    worktree: RecordedWorktree,
    identity: &'claim Identity,
    write: Write,
}

impl<'claim, Write: Fn() -> Result<String, CliFailure>> ClaimRun<'claim, Write> {
    pub fn new(
        clone: &'claim LocalClone,
        worktree: RecordedWorktree,
        identity: &'claim Identity,
        write: Write,
    ) -> Self {
        Self { clone, checkout: Git::from(clone), worktree, identity, write }
    }

    pub fn ran(&self, plan: &ClaimPlan) -> Result<String, CliFailure> {
        let steps = plan.steps();
        steps.iter().enumerate().try_fold(String::new(), |shown, (at, step)| {
            Ok(shown + &self.attempted(step, steps.get(..at).unwrap_or_default())?)
        })
    }

    pub fn created(&self) -> Result<PathBuf, CliFailure> {
        std::fs::canonicalize(self.worktree.path()).map_err(CliFailure::raised::<Setup>)
    }

    fn attempted(&self, step: &ClaimStep, done: &[ClaimStep]) -> Result<String, CliFailure> {
        step.taken(self).map_err(|failure| {
            ClaimPlan::rollback(done).into_iter().fold(failure, |failure, undo| undo.applied(self, failure))
        })
    }
}

impl<Write: Fn() -> Result<String, CliFailure>> ClaimHands for ClaimRun<'_, Write> {
    fn pruned(&self) -> Result<(), CliFailure> {
        self.checkout.output(GitArgv::WorktreePrune, &[]).map(drop)
    }

    fn configured(&self) -> Result<(), CliFailure> {
        self.checkout.output(GitArgv::WorktreeConfig, &[]).map(drop)
    }

    fn branched(&self, base: &BranchName) -> Result<(), CliFailure> {
        self.checkout.output(GitArgv::BranchNoTrack, &[self.worktree.branch().as_ref(), base.as_ref()]).map(drop)
    }

    fn added(&self) -> Result<(), CliFailure> {
        self.checkout
            .output(GitArgv::WorktreeAdd, &[self.worktree.path().as_os_str(), self.worktree.branch().as_ref()])
            .map(drop)
    }

    fn settled(&self) -> Result<(), CliFailure> {
        let worktree = Git::new(self.worktree.path(), &self.clone.settings);
        let git_dir = PathBuf::from(worktree.output(GitArgv::AbsoluteGitDir, &[])?);
        WorktreeIdentity::new(self.identity.name(), self.identity.email()).written(&git_dir)?;
        worktree.output(GitArgv::ResetHard, &[])?;
        let id = WorktreeId::parsed(&git_dir).ok_or_else(|| CliFailure::raised::<Setup>("no worktree id"))?;
        let recorded = self.worktree.clone().relocated(&self.created()?).identified(id);
        LocalTrail::append(self.clone.git_dir.as_ref(), &TrailEntry::Created(recorded))
    }

    fn written(&self) -> Result<String, CliFailure> {
        (self.write)()
    }

    fn untrailed(&self) -> Result<(), CliFailure> {
        let path = self.created().unwrap_or_else(|_| self.worktree.path().to_path_buf());
        LocalTrail::append(self.clone.git_dir.as_ref(), &TrailEntry::Removed { path, at: Utc::now() })
    }

    fn removed(&self) {
        self.checkout.holds(GitArgv::WorktreeRemove, &[self.worktree.path().as_os_str()]);
    }

    fn deleted(&self) {
        self.checkout.holds(GitArgv::BranchDelete, &[self.worktree.branch().as_ref()]);
    }
}
