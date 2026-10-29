use std::ffi::OsStr;
use std::path::{Path, PathBuf};

use chrono::Utc;
use meta_git::{
    BranchCreate, BranchDelete, ConfigFlag, Reads as _, Repository, Revision, WorktreeAdd, WorktreePrune,
    WorktreeRemove, Writes as _,
};

use super::LocalClone;
use super::claim_plan::{ClaimHands, ClaimPlan, ClaimRefusal, ClaimStep};
use crate::request::ci::{SessionId, SessionStart};
use crate::request::cli_failure::{CliFailure, Setup};
use crate::request::local::local_settings::Identity;
use crate::request::local::local_trail::LocalTrail;
use crate::request::local::recorded_worktree::RecordedWorktree;
use crate::request::local::trail_entry::TrailEntry;
use crate::request::local::worktree_id::WorktreeId;
use crate::request::project::BranchName;

pub struct ClaimRun<'claim, Write: Fn() -> Result<String, CliFailure>> {
    clone: &'claim LocalClone,
    checkout: Repository,
    worktree: RecordedWorktree,
    identity: &'claim Identity,
    write: Write,
}

impl<'claim, Write: Fn() -> Result<String, CliFailure>> ClaimRun<'claim, Write> {
    const WORKTREE_CONFIG: &'static str = "extensions.worktreeConfig";

    pub fn new(
        clone: &'claim LocalClone,
        worktree: RecordedWorktree,
        identity: &'claim Identity,
        write: Write,
    ) -> Result<Self, CliFailure> {
        Ok(Self { clone, checkout: clone.repository()?, worktree, identity, write })
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

    fn name(&self) -> String {
        self.worktree.path().file_name().map(OsStr::to_string_lossy).unwrap_or_default().into_owned()
    }

    fn branch(name: &BranchName) -> String {
        AsRef::<OsStr>::as_ref(name).to_string_lossy().into_owned()
    }
}

impl<Write: Fn() -> Result<String, CliFailure>> ClaimHands for ClaimRun<'_, Write> {
    fn pruned(&self) -> Result<(), CliFailure> {
        self.checkout.write(&WorktreePrune).map_err(CliFailure::raised::<Setup>)
    }

    fn configured(&self) -> Result<(), CliFailure> {
        self.checkout.write(&ConfigFlag::from(Self::WORKTREE_CONFIG.to_owned())).map_err(CliFailure::raised::<Setup>)
    }

    fn branched(&self, base: &BranchName) -> Result<(), CliFailure> {
        let found = self.checkout.read(&Revision::from(Self::branch(base))).map_err(CliFailure::raised::<Setup>)?;
        let at = found.ok_or_else(|| CliFailure::raised::<Setup>(ClaimRefusal::NoBase(base.clone())))?;
        let created = BranchCreate::new(Self::branch(self.worktree.branch()), at);
        self.checkout.write(&created).map_err(CliFailure::raised::<Setup>)
    }

    fn added(&self) -> Result<(), CliFailure> {
        let identity = self.identity.name().signing(self.identity.email());
        let path = self.worktree.path().to_path_buf();
        let added = WorktreeAdd::new(self.name(), path, Self::branch(self.worktree.branch()), identity);
        self.checkout.write(&added).map_err(CliFailure::raised::<Setup>)
    }

    fn settled(&self) -> Result<(), CliFailure> {
        let name = self.name();
        let id = WorktreeId::parsed(Path::new(&name)).ok_or_else(|| CliFailure::raised::<Setup>("no worktree id"))?;
        let recorded = self.worktree.clone().relocated(&self.created()?).identified(id);
        SessionStart::for_session(&SessionId::of_task(self.worktree.task()))
            .record()
            .map_err(CliFailure::raised::<Setup>)?;
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
        self.checkout.write(&WorktreeRemove::from(self.name())).ok();
    }

    fn deleted(&self) {
        self.checkout.write(&BranchDelete::from(Self::branch(self.worktree.branch()))).ok();
    }
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use meta_git::{Layout, Repository};
    use meta_schema::Id;

    use super::{ClaimHands, ClaimRun, Identity, LocalClone};
    use crate::request::local::local_trail::LocalTrail;
    use crate::request::local::recorded_worktree::RecordedWorktree;
    use crate::request::local::trail_entry::TrailEntry;
    use crate::request::project::{BranchName, CiSettings, Config};

    const NOWHERE: &str = "/nonexistent/clone";
    const GIT_DIR: &str = ".git";
    const BASE: &str = "base";
    const SHOWN: &str = "base: no such commit";
    const AGENT: Option<&str> = Some("a");
    const NOREPLY: Option<&str> = Some("1+a@users.noreply.github.com");
    const NAME: &str = "clone";
    const STALE: &str = "worktrees/stale";
    const STALE_FILES: [[&str; 2]; 3] =
        [["gitdir", "/nonexistent/stale/.git\n"], ["commondir", "../..\n"], ["HEAD", "ref: refs/heads/main\n"]];

    struct Scene;

    impl Scene {
        fn clone(checkout: &Path) -> LocalClone {
            Repository::created(checkout, Layout::Working).unwrap();
            let stale = checkout.join(GIT_DIR).join(STALE);
            std::fs::create_dir_all(&stale).unwrap();
            STALE_FILES.iter().try_for_each(|[file, text]| std::fs::write(stale.join(file), text)).unwrap();
            let settings = CiSettings::try_from(&Config::default()).unwrap();
            LocalClone::new(checkout.to_path_buf(), checkout.join(GIT_DIR), settings)
        }
    }

    #[test]
    fn an_untrailed_worktree_leaves_the_trail_and_an_unknown_base_is_named() {
        let checkout = tempfile::tempdir().unwrap();
        let clone = Scene::clone(checkout.path());
        let git_dir = checkout.path().join(GIT_DIR);
        let worktree = RecordedWorktree::new(PathBuf::from(NOWHERE), Id::generate());
        LocalTrail::append(&git_dir, &TrailEntry::Created(worktree.clone())).unwrap();
        let identity = Identity::from_git(AGENT, NOREPLY).unwrap();
        let run = ClaimRun::new(&clone, worktree, &identity, || Ok(String::new())).unwrap();
        let untrailed = run.untrailed().is_ok() && LocalTrail::read(&git_dir).unwrap().current().is_empty();
        let base: BranchName = BASE.parse().unwrap();
        let shown = run.branched(&base).unwrap_err().to_string();
        assert!(run.pruned().is_ok() && !git_dir.join(STALE).exists() && untrailed && run.name() == NAME);
        assert!(shown.contains(SHOWN), "{shown}");
    }
}
