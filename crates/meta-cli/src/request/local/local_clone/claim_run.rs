mod target_seed;
mod worktree_identity;

use std::path::PathBuf;

use chrono::Utc;
use meta_run::GitArgv;

use super::LocalClone;
use super::claim_plan::{ClaimHands, ClaimPlan, ClaimStep};
use super::git::Git;
use crate::request::ci::{SessionId, SessionStart};
use crate::request::cli_failure::{CliFailure, Setup};
use crate::request::local::local_settings::Identity;
use crate::request::local::local_trail::LocalTrail;
use crate::request::local::recorded_worktree::RecordedWorktree;
use crate::request::local::trail_entry::TrailEntry;
use crate::request::local::worktree_id::WorktreeId;
use crate::request::project::BranchName;
use target_seed::TargetSeed;
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
        SessionStart::for_session(&SessionId::of_task(self.worktree.task()))
            .record()
            .map_err(CliFailure::raised::<Setup>)?;
        LocalTrail::append(self.clone.git_dir.as_ref(), &TrailEntry::Created(recorded))
    }

    fn seeded(&self) -> Result<(), CliFailure> {
        TargetSeed::new(&self.clone.checkout, &self.created()?).sown()
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

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use meta_schema::Id;

    use super::{ClaimHands, ClaimRun, Git, GitArgv, Identity, LocalClone};
    use crate::request::local::local_trail::LocalTrail;
    use crate::request::local::recorded_worktree::RecordedWorktree;
    use crate::request::local::trail_entry::TrailEntry;
    use crate::request::project::{BranchName, CiSettings, Config};

    const NOWHERE: &str = "/nonexistent/clone";
    const SHOWN: &str = "git -C /nonexistent/clone branch --no-track task-";
    const AGENT: Option<&str> = Some("a");
    const NOREPLY: Option<&str> = Some("1+a@users.noreply.github.com");

    #[test]
    fn pruning_runs_git_in_the_checkout_and_an_untrailed_worktree_leaves_the_trail() {
        let git_dir = tempfile::tempdir().unwrap();
        let settings = CiSettings::try_from(&Config::default()).unwrap();
        let clone = LocalClone::new(PathBuf::from(NOWHERE), git_dir.path().to_path_buf(), settings);
        let worktree = RecordedWorktree::new(PathBuf::from(NOWHERE).join(NOWHERE), Id::generate());
        LocalTrail::append(git_dir.path(), &TrailEntry::Created(worktree.clone())).unwrap();
        let identity = Identity::from_git(AGENT, NOREPLY).unwrap();
        let run = ClaimRun::new(&clone, worktree, &identity, || Ok(String::new()));
        let untrailed = run.untrailed().is_ok() && LocalTrail::read(git_dir.path()).unwrap().current().is_empty();
        let base: BranchName = "base".parse().unwrap();
        let shown = run.branched(&base).unwrap_err().to_string();
        assert!(run.pruned().is_err() && untrailed && Git::from(&clone).holds(GitArgv::Head, &[]).is_none());
        assert!(shown.starts_with(SHOWN) && shown.contains(" base: "));
    }
}
